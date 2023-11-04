use clap::{ArgGroup, Parser, Subcommand};
use clap_num::maybe_hex;

use crate::tuner::Voltage;

#[derive(Debug, Parser)]
#[clap(name = "recisdb")]
#[clap(about = "recisdb can read both Unix chardev-based and BonDriver-based TV sources. ", long_about = None)]
#[clap(author = "maleicacid")]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Signal test.
    /// This subcommand tests the signal quality of the tuner
    /// and prints the S/N rate in dB.
    /// The signal quality is measured by the tuner's internal
    /// signal detector.
    #[clap(name = "checksignal")]
    Checksignal {
        /// The device name.
        /// This is the name of the device as specified in the
        /// `/dev/` directory.
        /// To use this option, you must specify the `-c` option.
        /// When the device is a BonDriver-based device,
        /// the name of the dll comes here.
        /// When the device is a Unix chardev-based device,
        /// the canonical path of the device comes here.
        #[clap(short, long, value_name = "CANONICAL_PATH", required = true)]
        device: String,

        /// The channel name.
        /// The channel name is a string that is defined in the
        /// `channels` module.
        #[clap(short, long, required = true)]
        channel: Option<String>,

        /// Override the transport stream ID(TSID) to obtain the stream (especially in ISDB-S w/ V4L DVB).
        #[clap(long, value_parser=maybe_hex::<u32>)]
        tsid: Option<u32>,

        /// LNB voltage.
        /// The LNB voltage is specified by the following flags.
        /// If none of the flags is specified, the LNB voltage is assumed unset.
        /// If multiple flags are specified, the highest voltage is assumed.
        #[clap(value_enum, long = "lnb")]
        lnb: Option<Voltage>,
    },
    /// Tune to a channel.
    /// This subcommand tunes the tuner to a channel and start recording.
    /// The channel is specified by a channel name.
    /// The channel name is a string that is defined in the
    /// `channels` module.
    /// The recording directory is passed as an argument.
    // key0 and key1 are optional, but if they are specified, they must be specified together
    #[clap(group(
    ArgGroup::new("key")
    .args(& ["key0", "key1"])
    .requires_all(& ["key0", "key1"])
    .multiple(true)
    ))]
    Tune {
        /// The device name.
        /// This is the name of the device as specified in the
        /// `/dev/` directory.
        /// To use this option, you must specify the `-c` option.
        /// When the device is a BonDriver-based device,
        /// the name of the dll comes here.
        /// When the device is a Unix chardev-based device,
        /// the canonical path of the device comes here.
        #[clap(short, long, value_name = "CANONICAL_PATH", required = true)]
        device: Option<String>,

        /// The channel name.
        /// The channel name is a string that is defined in the
        /// `channels` module.
        #[clap(short, long, required = true)]
        channel: Option<String>,

        /// Override the transport stream ID(TSID) to obtain the stream (especially in ISDB-S w/ V4L DVB).
        #[clap(long, value_parser=maybe_hex::<u32>)]
        tsid: Option<u32>,

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

        /// Continue on error when the decoding failed while processing.
        #[clap(short = 'k', long)]
        continue_on_error: bool,

        /// Disable ARIB STD-B25 decoding.
        /// If this flag is specified, ARIB STD-B25 decoding is not performed.
        #[clap(long = "no-decode")]
        no_decode: bool,
        /// Disable SIMD in MULTI2 processing.
        #[clap(long = "no-simd")]
        no_simd: bool,
        /// Disable null packet stripping.
        /// If this flag is specified, the decoder won't discard meaningless packets automatically.
        #[clap(long = "no-strip")]
        no_strip: bool,

        /// LNB voltage.
        /// The LNB voltage is specified by the following flags.
        /// If none of the flags is specified, the LNB voltage is assumed unset.
        /// If multiple flags are specified, the highest voltage is assumed.
        #[clap(value_enum, long = "lnb")]
        lnb: Option<Voltage>,

        /// The first working key (only available w/ "crypto" feature).
        /// The first working key is a 64-bit hexadecimal number.
        /// If the first working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key0")]
        key0: Option<Vec<String>>,
        /// The second working key (only available w/ "crypto" feature).
        /// The second working key is a 64-bit hexadecimal number.
        /// If the second working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key1")]
        key1: Option<Vec<String>>,

        /// The location of the output.
        /// The location is a string that is specified as an
        /// absolute path.
        /// If '-' is specified, the recording will be redirected to
        /// stdout.
        /// If the specified file is a directory, this subcommand
        /// will stop.
        #[clap(required = true)]
        output: Option<String>,
    },
    /// Perform ARIB STD-B25 decoding on TS stream.
    #[clap(group(
    ArgGroup::new("key")
    .args(& ["key0", "key1"])
    .requires_all(& ["key0", "key1"])
    .multiple(true)
    ))]
    Decode {
        /// The source file name.
        /// The source file name is a string that is specified as a
        /// file name.
        /// If '--device' is specified, this parameter is ignored.
        #[clap(short = 'i', long = "input", value_name = "file", required = true)]
        source: Option<String>,

        /// Disable SIMD in MULTI2 processing.
        #[clap(long = "no-simd")]
        no_simd: bool,
        /// Disable null packet stripping.
        /// If this flag is specified, the decoder won't discard meaningless packets automatically.
        #[clap(long = "no-strip")]
        no_strip: bool,

        /// The first working key (only available w/ "crypto" feature).
        /// The first working key is a 64-bit hexadecimal number.
        /// If the first working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key0")]
        key0: Option<Vec<String>>,
        /// The second working key (only available w/ "crypto" feature).
        /// The second working key is a 64-bit hexadecimal number.
        /// If the second working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key1")]
        key1: Option<Vec<String>>,

        /// The location of the output.
        /// The location is a string that is specified as an
        /// absolute path.
        /// If '-' is specified, the recording will be redirected to
        /// stdout.
        /// If the specified file is a directory, this subcommand
        /// will stop.
        #[clap(required = true)]
        output: Option<String>,
    },
}
