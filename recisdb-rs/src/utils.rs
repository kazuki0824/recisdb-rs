use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use env_logger::Env;
use futures_util::io::{AllowStdIo, BufReader};
use log::{error, info};

use b25_sys::futures_io::AsyncBufRead;

use crate::channels;
use crate::tuner_base::{Tuned, Voltage};

pub(crate) fn get_src(
    device: Option<String>,
    channel: Option<channels::Channel>,
    source: Option<String>,
    voltage: Option<Voltage>,
) -> Result<Box<dyn AsyncBufRead + Unpin>, Box<dyn Error>> {
    if let Some(src) = device {
        crate::tuner_base::tune(&src, channel.unwrap(), voltage).map(|tuned| tuned.open_stream())
    } else if let Some(src) = source {
        let src = std::fs::canonicalize(src)?;
        let input = BufReader::with_capacity(20000, AllowStdIo::new(std::fs::File::open(src)?));
        Ok(Box::new(input) as Box<dyn AsyncBufRead + Unpin>)
    } else {
        info!("Waiting for stdin...");
        let input = BufReader::with_capacity(20000, AllowStdIo::new(std::io::stdin().lock()));
        Ok(Box::new(input) as Box<dyn AsyncBufRead + Unpin>)
    }
}

pub(crate) fn get_output(directory: Option<String>) -> Result<Box<dyn Write>, std::io::Error> {
    match directory {
        Some(s) if s == "-" => Ok(Box::new(std::io::stdout().lock()) as Box<dyn Write>),
        None => {
            let filename_time_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(Box::new(fs::File::create(format!(
                "{}/{}.m2ts",
                env::current_dir()?.to_str().unwrap(),
                filename_time_now
            ))?))
        }
        Some(path) if Path::exists(Path::new(&path)) => {
            let path = fs::canonicalize(path)?;
            if fs::metadata(&path)?.is_file() {
                Ok(Box::new(fs::File::create(path)?))
            } else {
                panic!("The file is directory")
            }
        }
        Some(path) => {
            let path = Path::new(&path);
            let path = PathBuf::from(path);
            match path.parent() {
                None => unreachable!("Unknown parent directory"),
                Some(dir) if dir.exists() => Ok(Box::new(fs::File::create(path)?)),
                _ => {
                    error!("Parent directory not found");
                    panic!()
                }
            }
        }
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
        _ => panic!("Specify both of the keys"),
    }
}

pub(crate) fn initialize_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}
