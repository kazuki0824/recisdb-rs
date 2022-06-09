use clap::{ArgGroup, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "recisdb-rs")]
#[clap(about = "Recisdb-rs can read both Unix chardev-based and BonDriver-based TV sources. ", long_about = None)]
#[clap(author = "maleicacid")]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Signal test
    /// This subcommand tests the signal quality of the tuner
    /// and prints the S/N rate in dB.
    /// The signal quality is measured by the tuner's internal
    /// signal detector.
    #[clap(name = "checksignal")]
    Checksignal {
        /// The device name
        /// This is the name of the device as specified in the
        /// `/dev/` directory.
        #[clap(short, long, required = true, value_name = "canonical_path")]
        device: String,
        /// The channel name
        /// The channel name is a string that is defined in the
        /// `channels` module.
        #[clap(short)]
        channel: Option<String>,
    },
    /// Tune to a channel
    /// This subcommand tunes the tuner to a channel and start recording.
    /// The channel is specified by a channel name.
    /// The channel name is a string that is defined in the
    /// `channels` module.
    /// The recording directory is passed as an argument.
    #[clap(name = "tune")]
    #[clap(group(
    ArgGroup::new("control")
    .args(&["time", "channel"])
    .requires_all(&["channel"])
    .requires("device")
    ))]
    //key0 and key1 are optional, but if they are specified, they must be specified together
    #[clap(group(
    ArgGroup::new("key")
    .args(&["key0", "key1"])
    .requires_all(&["key0", "key1"])
    .multiple(true)
    ))]
    #[clap(group(
    ArgGroup::new("input")
    .multiple(false)
    .args(&["device", "source"])
    ))]
    Tune {
        /// The device name
        /// This is the name of the device as specified in the
        /// `/dev/` directory.
        /// To use this option, you must specify the `-c` option.
        /// When the device is a BonDriver-based device,
        /// the name of the dll comes here.
        /// When the device is a Unix chardev-based device,
        /// the canonical path of the device comes here.
        /// If the device name is not specified, this subcommand will try
        /// to read the data from the specified file.
        #[clap(short, long, value_name = "canonical_path")]
        device: Option<String>,
        /// The source file name
        /// The source file name is a string that is specified as a
        /// file name.
        /// If the device name is not specified, this subcommand will
        /// try to read the data from the specified data source.
        /// If '--device' is specified, this parameter is ignored.
        #[clap(short = 'i', long = "input", value_name = "file")]
        source: Option<String>,

        /// The channel name
        /// The channel name is a string that is defined in the
        /// `channels` module.
        #[clap(short)]
        channel: Option<String>,
        /// The duration of the recording
        /// The duration of the recording is specified in seconds.
        /// If the duration is not specified, the recording will
        /// continue until the user stops it.
        /// The duration is specified as a floating point number.
        /// If the duration is 0.0, the recording will continue
        /// until the user stops it.
        /// If the duration is negative, the recording will
        /// continue until the user stops it.
        /// If the duration is positive, the recording will
        /// continue until the duration is over.
        #[clap(short, long, value_name = "seconds")]
        time: Option<f64>,

        /// The first working key
        /// The first working key is a 64-bit hexadecimal number.
        /// If the first working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(short = 'k', long = "key0")]
        key0: Option<String>,
        /// The second working key
        /// The second working key is a 64-bit hexadecimal number.
        /// If the second working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(short = 'K', long = "key1")]
        key1: Option<String>,

        /// The recording directory
        /// The recording directory is a string that is specified as a
        /// directory name.
        /// If the recording directory is not specified, the recording
        /// will be stored in the current directory.
        /// If '-' is specified, the recording will be redirected to
        /// stdout.
        /// If the specified directory does not exist, this subcommand
        /// will stop.
        #[clap(required = true)]
        directory: Option<String>,
    },
}
