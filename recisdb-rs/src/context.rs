use clap::{ArgGroup, Parser, Subcommand};
use clap_num::maybe_hex;

use crate::tuner::Voltage;

#[derive(Debug, Parser)]
#[clap(name = "recisdb")]
#[clap(about = "recisdb can read both Unix chardev-based and BonDriver-based TV sources. ", long_about = None)]
#[clap(author = "maleicacid")]
#[clap(version)]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Signal test.{n}
    /// This subcommand tests the signal quality of the tuner
    /// and prints the S/N rate in dB.{n}
    /// The signal quality is measured by the tuner's internal
    /// signal detector.
    #[clap(name = "checksignal")]
    Checksignal {
        /// The device name.{n}
        /// This is the name of the device as specified in the
        /// `/dev/` directory.{n}
        /// To use this option, you must specify the `-c` option.{n}
        /// When the device is a BonDriver-based device,
        /// the name of the dll comes here.{n}
        /// When the device is a Unix chardev-based device,
        /// the canonical path of the device comes here.{n}
        /// If the device has a V4L-DVB interface, there are 2 ways to point the frontend.{n}
        /// 1. (full) `-c /dev/dvb/adapter2/frontend0`{n}
        /// 2. (abbr.) `-c "2|0"`
        #[clap(short, long, value_name = "CANONICAL_PATH", required = true)]
        device: String,

        /// The channel name.{n}
        /// The channel name is a string that is defined in the
        /// `channels` module.
        #[clap(short, long, required = true)]
        channel: Option<String>,

        /// LNB voltage.
        /// If none, the LNB voltage is assumed unset.{n}
        #[clap(value_enum, long = "lnb")]
        lnb: Option<Voltage>,
    },
    /// Tune to a channel.
    /// This subcommand tunes the tuner to a channel and start recording.{n}
    /// The channel is specified by a channel name.{n}
    /// The recording directory is passed as an argument.
    // key0 and key1 are optional, but if they are specified, they must be specified together
    #[clap(group(
    ArgGroup::new("key")
    .args(& ["key0", "key1"])
    .requires_all(& ["key0", "key1"])
    .multiple(true)
    ))]
    Tune {
        /// The device name.{n}
        /// This is the name of the device as specified in the
        /// `/dev/` directory.{n}
        /// To use this option, you must specify the `-c` option.{n}
        /// When the device is a BonDriver-based device,
        /// the name of the DLL comes here.{n}
        /// When the device is a Unix chardev-based device,
        /// the canonical path of the device comes here.{n}
        /// If the device has a V4L-DVB interface, there are 2 ways to point the frontend.{n}
        /// 1. (full) `-c /dev/dvb/adapter2/frontend0`{n}
        /// 2. (abbr.) `-c "2|0"`
        #[clap(short = 'i', long, value_name = "CANONICAL_PATH", required = true)]
        device: Option<String>,

        /// The channel name.{n}
        /// The channel name is a string that is defined in the
        /// `channels` module.
        #[clap(short, long, required = true)]
        channel: Option<String>,

        /// The card reader name.
        #[clap(long)]
        card: Option<String>,

        /// Override the transport stream ID(TSID) to obtain the stream (especially in ISDB-S w/ V4L-DVB).
        #[clap(long, value_parser=maybe_hex::<u32>)]
        tsid: Option<u32>,

        /// The duration of the recording.{n}
        /// The duration of the recording is specified in seconds.
        /// If the duration is not specified, the recording will
        /// continue until the user stops it.{n}
        /// The duration is specified as a floating point number.{n}
        /// If the duration is 0.0, the recording will continue
        /// until the user stops it.
        /// If the duration is negative, the recording will
        /// continue until the user stops it.
        /// If the duration is positive, the recording will
        /// continue until the duration is over.
        #[clap(short, long, value_name = "seconds")]
        time: Option<f64>,

        /// Exit if the decoding fails while processing.
        #[clap(short = 'e', long)]
        exit_on_card_error: bool,

        /// Disable ARIB STD-B25 decoding.{n}
        /// If this flag is specified, ARIB STD-B25 decoding is not performed.
        #[clap(long = "no-decode")]
        no_decode: bool,
        /// Disable SIMD in MULTI2 processing.
        #[clap(long = "no-simd")]
        no_simd: bool,
        /// Disable null packet stripping.{n}
        /// If this flag is specified, the decoder won't discard meaningless packets automatically.
        #[clap(long = "no-strip")]
        no_strip: bool,

        /// LNB voltage.
        /// If none, the LNB voltage is assumed unset.{n}
        #[clap(value_enum, long = "lnb")]
        lnb: Option<Voltage>,

        /// The first working key (only available w/ "crypto" feature).{n}
        /// The first working key is a 64-bit hexadecimal number.{n}
        /// If the first working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key0")]
        key0: Option<Vec<String>>,
        /// The second working key (only available w/ "crypto" feature).{n}
        /// The second working key is a 64-bit hexadecimal number.{n}
        /// If the second working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key1")]
        key1: Option<Vec<String>>,

        /// The location of the output.{n}
        /// The location is a string that is specified as an
        /// absolute path.{n}
        /// If '-' is specified, the recording will be redirected to
        /// stdout.{n}
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
        /// The source file name.{n}
        /// The source file name is a string that is specified as a
        /// file name.{n}
        /// If '--device' is specified, this parameter is ignored.
        #[clap(short = 'i', long = "input", value_name = "file", required = true)]
        source: Option<String>,

        /// Disable SIMD in MULTI2 processing.
        #[clap(long = "no-simd")]
        no_simd: bool,
        /// Disable null packet stripping.{n}
        /// If this flag is specified, the decoder won't discard meaningless packets automatically.
        #[clap(long = "no-strip")]
        no_strip: bool,

        /// The card reader name.
        #[clap(long)]
        card: Option<String>,

        /// The first working key (only available w/ "crypto" feature).{n}
        /// The first working key is a 64-bit hexadecimal number.{n}
        /// If the first working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key0")]
        key0: Option<Vec<String>>,
        /// The second working key (only available w/ "crypto" feature).{n}
        /// The second working key is a 64-bit hexadecimal number.{n}
        /// If the second working key is not specified, this subcommand
        /// will not decode ECM.
        #[clap(long = "key1")]
        key1: Option<Vec<String>>,

        /// The location of the output.{n}
        /// The location is a string that is specified as an
        /// absolute path.{n}
        /// If '-' is specified, the recording will be redirected to
        /// stdout.{n}
        /// If the specified file is a directory, this subcommand
        /// will stop.
        #[clap(required = true)]
        output: Option<String>,
    },
    #[cfg(windows)]
    Enumerate {
        #[clap(short = 'i', long, value_name = "CANONICAL_PATH", required = true)]
        device: String,
        #[clap(short, long, required = true)]
        space: u32,
    },
}
