use dvbv5::FilePtr;
use dvbv5_sys::dvb_file_formats;
use dvbv5_sys::fe_delivery_system;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

fn seek(input: FilePtr, ch_name: &str) -> Option<u32> {
    for (_index, entry) in input.iter().enumerate() {
        match entry.get_channel() {
            Ok(val) if val == ch_name => {
                return match dvbv5::retrieve_entry_prop(
                    &entry,
                    dvbv5::dtv_retrievable_properties::DTV_STREAM_ID,
                ) {
                    Ok(id) => Some(id),
                    Err(_) => None,
                };
            }
            _ => continue,
        }
    }
    None
}

/// Look up the TSID (STREAM_ID) for the given channel name from the
/// hardcoded ISDB-S table (`dvbv5_channels_isdbs.conf`).
///
/// This is used for both BS relative-TS-number lookups (e.g. `BS03_0`)
/// and CS transponder lookups (e.g. `CS2`).
///
/// A mutex serialises access because libdvbv5's file parser is not
/// guaranteed to be thread-safe.
pub(crate) fn lookup_tsid(ch_name: &str) -> Option<u32> {
    static LOCK: Mutex<()> = Mutex::new(());

    const CONF: &str = include_str!("./dvbv5_channels_isdbs.conf");

    // Use a unique file path per call to avoid races when multiple
    // threads perform lookups concurrently.
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let path = format!(
        "/tmp/dvbv5_channels_isdbs_{}_{}.conf",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );

    let _guard = LOCK.lock().ok()?;
    fs::write(&path, CONF).ok()?;
    let entries = FilePtr::new(
        Path::new(&path),
        Some(fe_delivery_system::SYS_ISDBS),
        Some(dvb_file_formats::FILE_DVBV5),
    )
    .ok()?;
    let result = seek(entries, ch_name);
    let _ = fs::remove_file(&path);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_cs_tsid() {
        assert_eq!(lookup_tsid("CS2"), Some(24608));
        assert_eq!(lookup_tsid("CS4"), Some(28736));
        assert_eq!(lookup_tsid("CS8"), Some(24704));
        assert_eq!(lookup_tsid("CS24"), Some(29056));
    }

    #[test]
    fn lookup_bs_tsid() {
        assert_eq!(lookup_tsid("BS03_0"), Some(16432));
        assert_eq!(lookup_tsid("BS01_0"), Some(16400));
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert_eq!(lookup_tsid("CS99"), None);
        assert_eq!(lookup_tsid("BS99_0"), None);
    }
}
