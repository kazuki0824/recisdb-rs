use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;

use futures_util::io::{AllowStdIo, BufReader};
use futures_util::AsyncBufRead;
use log::info;

use crate::channels;
use crate::tuner::{Tunable, UnTunedTuner, Voltage};

pub(crate) fn get_src(
    device: Option<String>,
    channel: Option<channels::Channel>,
    source: Option<String>,
    voltage: Option<Voltage>,
) -> Result<Box<dyn AsyncBufRead + Unpin>, Box<dyn Error>> {
    if let Some(src) = device {
        Ok(Box::new(
            UnTunedTuner::new(src)?.tune(channel.unwrap(), voltage)?,
        ))
    } else if let Some(src) = source {
        if src == "-" {
            info!("Waiting for stdin...");
            let input = BufReader::with_capacity(20000, AllowStdIo::new(std::io::stdin().lock()));
            return Ok(Box::new(input) as Box<dyn AsyncBufRead + Unpin>);
        }
        let src = fs::canonicalize(src)?;
        let input = BufReader::with_capacity(20000, AllowStdIo::new(fs::File::open(src)?));
        Ok(Box::new(input) as Box<dyn AsyncBufRead + Unpin>)
    } else {
        unreachable!("Either device & channel or source must be specified.")
    }
}

pub(crate) fn get_output(path: Option<String>) -> Result<Box<dyn Write>, std::io::Error> {
    match path {
        Some(s) if s == "-" => Ok(Box::new(std::io::stdout().lock()) as Box<dyn Write>),
        Some(s) if s == "/dev/null" => Ok(Box::new(fs::File::create(s)?)),
        Some(path) => {
            let p = Path::new(&path);
            let path_buf;
            // If the path already exists, it could be a file or directory.
            if p.exists() {
                if p.is_file() {
                    // If it is a file, we will write to this file.
                    // e.g. "/existing/path/to/file.txt"
                    return Ok(Box::new(fs::File::create(p)?));
                } else {
                    // If it is a directory, we will create a new file in this directory later.
                    // e.g. "/existing/path/to/directory"
                    path_buf = p.to_path_buf();
                }
            } else {
                // If the path does not exist, it could be a directory or a file that we want to create.
                // If it ends with a "/" or "\", we will consider it as a directory.
                // e.g. "/nonexisting/path/to/directory/" or "C:\nonexisting\path\to\directory\"
                if path.ends_with('/') || (cfg!(windows) && path.ends_with('\\')) {
                    fs::create_dir_all(&path)?;
                    path_buf = p.to_path_buf();
                    // If it does not end with a "/" or "\", we will consider it as a file.
                    // e.g. "/nonexisting/path/to/file.txt"
                } else {
                    let parent = p.parent().ok_or(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid path",
                    ))?;
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                    return Ok(Box::new(fs::File::create(p)?));
                }
            }
            // If the path is a directory, we will create a new file with the UNIX epoch time as the filename in this directory.
            let filename_time_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(Box::new(fs::File::create(format!(
                "{}/{}.m2ts",
                path_buf.to_str().unwrap(),
                filename_time_now
            ))?))
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No output path specified.",
        )),
    }
}

pub(crate) fn parse_keys(key0: Option<Vec<String>>, key1: Option<Vec<String>>) -> bool {
    //Parse and store keys and if configuration is valid, return true.
    match (key0, key1) {
        (None, None) => false,
        (Some(k0), Some(k1)) => {
            let k0 = k0
                .iter()
                .map(|k| u64::from_str_radix(k.trim_start_matches("0x"), 16).unwrap())
                .collect::<Vec<u64>>();
            let k1 = k1
                .iter()
                .map(|k| u64::from_str_radix(k.trim_start_matches("0x"), 16).unwrap())
                .collect::<Vec<u64>>();
            b25_sys::set_keys(k0, k1);
            true
        }
        _ => panic!("Specify both of the keys."),
    }
}
