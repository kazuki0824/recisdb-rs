use dvbv5::FilePtr;
use dvbv5_sys::dvb_file_formats;
use dvbv5_sys::fe_delivery_system;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub(crate) fn seek(input: FilePtr, ch_name: &str) -> Option<u32> {
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

pub(crate) fn get_tsid_tables() -> (FilePtr, FilePtr) {
    let name_s = "dvbv5_channels_isdbs.conf";
    let name_t = "dvbv5_channels_isdbt.conf";

    let s = include_str!("./dvbv5_channels_isdbs.conf");
    let t = include_str!("./dvbv5_channels_isdbt.conf");

    let path_s = format!("/tmp/{name_s}");
    let path_t = format!("/tmp/{name_t}");

    write!(File::create(&path_s).unwrap(), "{}", s).unwrap();
    write!(File::create(&path_t).unwrap(), "{}", t).unwrap();

    let s = FilePtr::new(
        Path::new(&path_s),
        Some(fe_delivery_system::SYS_ISDBS),
        Some(dvb_file_formats::FILE_DVBV5),
    )
    .unwrap();
    let t = FilePtr::new(
        Path::new(&path_t),
        Some(fe_delivery_system::SYS_ISDBS),
        Some(dvb_file_formats::FILE_DVBV5),
    )
    .unwrap();

    (s, t)
}
