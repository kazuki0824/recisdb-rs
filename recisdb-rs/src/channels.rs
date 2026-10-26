use crate::channels::representation::TsFilter::{AbsTsId, AsIs, RelTsNum};
pub(crate) use crate::channels::representation::{ChannelSpace, ChannelType};
use log::{error, warn};

pub mod output {
    use crate::channels::representation::{ChannelType, TsFilter};
    use log::error;

    #[repr(C)]
    #[allow(dead_code)]
    pub struct IoctlFreq {
        pub ch: i32,
        pub slot: i32,
    }

    impl From<ChannelType> for IoctlFreq {
        fn from(value: ChannelType) -> Self {
            const OFFSET_HZ: i32 = 0;

            let ioctl_channel = match &value {
                ChannelType::Terrestrial(ch_num, ..) if (13..=62).contains(ch_num) => ch_num + 50,
                ChannelType::Catv(ch_num, ..) if (23..=63).contains(ch_num) => ch_num - 1,
                ChannelType::Catv(ch_num, ..) if (13..=22).contains(ch_num) => ch_num - 10,
                ChannelType::CS(ch_num, ..) if (2..=24).contains(ch_num) && (ch_num % 2 == 0) => {
                    ch_num / 2 + 11
                }
                ChannelType::BS(ch_num, _) if (1..=23).contains(ch_num) && (ch_num % 2 == 1) => {
                    ch_num / 2
                }

                ChannelType::BonCh(_) | ChannelType::BonChSpace(_) => {
                    error!("{:?} is not supported in Linux.", value);
                    std::process::exit(-1)
                }
                _ => unreachable!("Invalid channel."),
            };
            let slot = match value {
                ChannelType::CS(_, TsFilter::AbsTsId(stream_id)) => stream_id as i32,
                ChannelType::CS(..) => 0,
                ChannelType::BS(_, TsFilter::AsIs) => -1,
                ChannelType::BS(_, TsFilter::AbsTsId(stream_id)) => stream_id as i32,
                ChannelType::BS(_, TsFilter::RelTsNum(num)) => num,

                _ => OFFSET_HZ,
            };

            Self {
                ch: ioctl_channel as i32,
                slot,
            }
        }
    }

    pub struct DvbFreq {
        pub freq_hz: u32,
        pub stream_id: Option<u32>,
    }

    impl From<ChannelType> for DvbFreq {
        fn from(value: ChannelType) -> Self {
            let freq: IoctlFreq = value.clone().into();

            let hz: u32 = match &value {
                ChannelType::Terrestrial(ch_num, ..) => {
                    /* UHF 13-62ch */
                    let ch_num = *ch_num as u32;
                    473142857 + (ch_num - 13) * 6000000
                }
                ChannelType::Catv(ch_num, ..) => {
                    /* CATV C13-C22ch */
                    let ch_num = *ch_num as u32;
                    if ch_num >= 13 && ch_num <= 22 {
                        111142857 + (ch_num - 13) * 6000000
                    /* CATV C23-C63ch */
                    } else if ch_num >= 23 && ch_num <= 63 {
                        225142857 + (ch_num - 23) * 6000000
                    } else {
                        unreachable!()
                    }
                }
                ChannelType::BS(..) | ChannelType::CS(..) => {
                    if freq.ch < 0 {
                        unreachable!()
                    } else if freq.ch < 12 {
                        /* BS */
                        (1049480 + (38360 * freq.ch)) as u32
                    } else if freq.ch < 24 {
                        /* CS */
                        (1613000 + (40000 * (freq.ch - 12))) as u32
                    } else {
                        unreachable!()
                    }
                }

                ChannelType::BonCh(_) | ChannelType::BonChSpace(_) => {
                    error!("{:?} is not supported in Linux.", value);
                    std::process::exit(-1)
                }
                _ => unreachable!("Invalid channel."),
            };

            let stream_id = match value {
                ChannelType::Terrestrial(_, TsFilter::AsIs)
                | ChannelType::Catv(_, TsFilter::AsIs) => None,
                ChannelType::BS(_, TsFilter::AsIs) | ChannelType::CS(_, TsFilter::AsIs) => None,
                ChannelType::BS(_, TsFilter::AbsTsId(id))
                | ChannelType::CS(_, TsFilter::AbsTsId(id)) => Some(id),
                ChannelType::BS(_, TsFilter::RelTsNum(id)) if id < 12 => Some(id as u32),
                _ => unreachable!(),
            };

            Self {
                freq_hz: hz,
                stream_id,
            }
        }
    }
}

pub mod representation {
    use std::fmt::Display;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum CableChannel {
        Catv(u8),
        VhfUhf(u8),
    }

    impl CableChannel {
        pub fn frequency_khz(&self) -> u32 {
            // px4_drv の PTX_SET_CHANNEL と同じ周波数番号に変換する
            let freq_no = match *self {
                Self::VhfUhf(ch @ 1..=3) => ch - 1,
                Self::Catv(ch @ 13..=22) => ch - 10,
                Self::VhfUhf(ch @ 4..=12) => ch + 9,
                Self::Catv(ch @ 23..=63) => ch - 1,
                Self::VhfUhf(ch @ 13..=62) => ch + 50,
                _ => unreachable!("Invalid cable channel."),
            } as u32;
            match freq_no {
                12..=16 => 93000 + freq_no * 6000 + 2000,
                0..=62 => 93000 + freq_no * 6000,
                _ => 95000 + freq_no * 6000,
            }
        }
    }

    /// J.83 Annex B (スカパー！プレミアムサービス光) の H01〜H77 の中心周波数 (kHz)
    /// 地域によってはこの 6.5MHz 刻みからずれた周波数で送られるため、周波数での指定も受け付ける
    pub fn j83b_frequency_khz(ch: u32) -> u32 {
        93250 + (ch - 1) * 6500
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ChannelSpace {
        pub space: u32,
        pub ch: u32,
        pub space_description: Option<String>,
        pub ch_description: Option<String>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum TsFilter {
        RelTsNum(i32),
        AbsTsId(u32),
        AsIs,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum ChannelType {
        Terrestrial(u8, TsFilter),
        Catv(u8, TsFilter),
        BS(u8, TsFilter),
        CS(u8, TsFilter),
        /// 物理チャンネルと、TSMF のヘッダーに載っている有効な TS の何本目か (1〜15)
        ISDBC(CableChannel, u8),
        /// 中心周波数 (kHz)
        J83B(u32),
        BonCh(u8),
        BonChSpace(ChannelSpace),
        Undefined,
    }

    impl Display for ChannelType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            type T = ChannelType;
            match self {
                T::Terrestrial(ch, TsFilter::AsIs) => write!(f, "Terrestrial: {ch}"),
                T::Catv(ch, TsFilter::AsIs) => write!(f, "CATV: {ch}"),
                T::Terrestrial(ch, tsid) => write!(f, "Terrestrial: {} (TS Filter={:?})", ch, tsid),
                T::Catv(ch, tsid) => write!(f, "CATV: {} (TS Filter={:?})", ch, tsid),
                T::BS(ch, tsid) => write!(f, "BS: {}, {:?}", ch, tsid),
                T::CS(ch, tsid) => write!(f, "CS: {}, {:?}", ch, tsid),
                T::ISDBC(ch, stream) => write!(f, "ISDB-C: {:?} (TSMF Stream={})", ch, stream),
                T::J83B(freq) => write!(f, "J.83 Annex B: {freq} kHz"),
                T::BonCh(ch) => write!(f, "BonDriver: Ch={ch}"),
                T::BonChSpace(ChannelSpace { ch, space, .. }) => {
                    write!(f, "BonDriver: Ch={ch}, Space={space}")
                }
                _ => write!(f, "Undefined"),
            }
        }
    }
}

mod parser {
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_while_m_n};
    use nom::character::complete::{u32, u8};
    use nom::combinator::{all_consuming, map_res, opt};
    use nom::sequence::{pair, preceded, separated_pair};
    use nom::{IResult, Parser};

    use super::representation::{j83b_frequency_khz, CableChannel};

    pub(crate) fn get_result(input: &str) -> IResult<&str, &str> {
        alt((
            tag("ISDBC-"),
            tag("J83B-"),
            tag("BS"),
            tag("CS"),
            tag("T"),
            tag("C"),
        ))
        .parse(input)
    }

    /// `C52`・`4`・`20`・`C52_2` のような ISDB-C の物理チャンネルと、省略できる `_<TSMF の何本目>`
    pub(crate) fn parse_isdbc(input: &str) -> IResult<&str, (CableChannel, u8)> {
        all_consuming(pair(
            alt((
                preceded(tag("C"), u8).map(CableChannel::Catv),
                u8.map(CableChannel::VhfUhf),
            )),
            opt(preceded(tag("_"), u8)).map(|stream| stream.unwrap_or(1)),
        ))
        .parse(input)
    }

    /// `H01`〜`H77` (2桁固定) か、中心周波数 (kHz) を受け付け、中心周波数 (kHz) を返す
    pub(crate) fn parse_j83b(input: &str) -> IResult<&str, u32> {
        all_consuming(alt((
            preceded(
                tag("H"),
                map_res(
                    take_while_m_n(2, 2, |c: char| c.is_ascii_digit()),
                    |digits: &str| match digits.parse::<u32>() {
                        Ok(ch @ 1..=77) => Ok(j83b_frequency_khz(ch)),
                        _ => Err(()),
                    },
                ),
            ),
            u32,
        )))
        .parse(input)
    }

    pub(crate) fn parse_integer_pair(input: &str) -> IResult<&str, (u32, u32)> {
        separated_pair(u32, alt((tag("-"), tag("_"))), u32).parse(input)
    }
}

pub struct Channel {
    pub ch_type: ChannelType,
    raw_string: String,
}

impl Channel {
    #[allow(unused)]
    pub fn get_raw_ch_name(&self) -> &str {
        self.raw_string.as_str()
    }

    pub fn new(ch_str: impl Into<String>, override_stream_id: Option<u32>) -> Self {
        let raw_string = ch_str.into();

        let ch_type = if let Ok(val) = raw_string.parse::<u8>() {
            ChannelType::BonCh(val)
        } else {
            // Parse
            match parser::get_result(&raw_string) {
                Ok((bottom, system @ ("ISDBC-" | "J83B-"))) => {
                    if override_stream_id.is_some() {
                        error!("--tsid cannot be used with ISDB-C/J.83B. Use _<n> (e.g. ISDBC-C52_2) to select a TSMF stream.");
                        ChannelType::Undefined
                    } else if system == "ISDBC-" {
                        match parser::parse_isdbc(bottom) {
                            Ok((_, (physical, stream))) => ChannelType::ISDBC(physical, stream),
                            Err(_) => ChannelType::Undefined,
                        }
                    } else {
                        match parser::parse_j83b(bottom) {
                            Ok((_, freq)) => ChannelType::J83B(freq),
                            Err(_) => ChannelType::Undefined,
                        }
                    }
                }
                Ok((bottom, "BS")) => {
                    if let Ok(ch) = bottom.parse() {
                        match override_stream_id {
                            None => ChannelType::BS(ch, AsIs),
                            Some(id) => ChannelType::BS(ch, AbsTsId(id)),
                        }
                    } else {
                        match (parser::parse_integer_pair(bottom), override_stream_id) {
                            (Ok((_, (first, _))), Some(id)) => {
                                ChannelType::BS(first as u8, AbsTsId(id))
                            }
                            (Ok((_, (first, second))), None) => {
                                ChannelType::BS(first as u8, RelTsNum(second as i32))
                            }

                            (Err(_), _) => ChannelType::Undefined,
                        }
                    }
                }
                Ok((bottom, "CS")) => match (bottom.parse(), override_stream_id) {
                    (Ok(ch), Some(id)) => ChannelType::CS(ch, AbsTsId(id)),
                    (Ok(ch), None) => ChannelType::CS(ch, AsIs),

                    (Err(_), _) => ChannelType::Undefined,
                },
                Ok((bottom, "C")) if override_stream_id.is_none() => {
                    if let Ok(ch) = bottom.parse() {
                        ChannelType::Catv(ch, AsIs)
                    } else {
                        ChannelType::Undefined
                    }
                }
                Ok((bottom, "T")) if override_stream_id.is_none() => {
                    if let Ok(ch) = bottom.parse() {
                        ChannelType::Terrestrial(ch, AsIs)
                    } else {
                        ChannelType::Undefined
                    }
                }
                _ => match parser::parse_integer_pair(&raw_string) {
                    Ok((_, (first, second))) => ChannelType::BonChSpace(ChannelSpace {
                        space: first,
                        ch: second,
                        space_description: None,
                        ch_description: None,
                    }),
                    Err(_) => ChannelType::Undefined,
                },
            }
        };

        let assert_range = |from, to, ch: &u8| {
            if !(from..=to).contains(ch) {
                error!("Channel value {ch} out of range. It must be from 13 to 63.")
            }
            (from..=to).contains(ch)
        };
        // Verify
        let consistent = match &ch_type {
            ChannelType::ISDBC(ch, stream) => {
                let physical_valid = match ch {
                    representation::CableChannel::Catv(ch) => (13..=63).contains(ch),
                    representation::CableChannel::VhfUhf(ch) => (1..=62).contains(ch),
                };
                if !physical_valid {
                    error!("ISDB-C requires CATV C13-C63 or VHF/UHF 1-62.");
                }
                let stream_valid = (1..=15).contains(stream);
                if !stream_valid {
                    error!("The TSMF stream number must be from 1 to 15.");
                }
                physical_valid && stream_valid
            }
            ChannelType::J83B(freq) => {
                // px4_drv の ChSet と同じく、VHF 1ch から UHF 62ch までの帯域を受け付ける
                let valid = (90000..=770000).contains(freq);
                if !valid {
                    error!(
                        "J.83B requires H01-H77 or a center frequency from 90000 to 770000 kHz."
                    );
                }
                valid
            }
            ChannelType::Terrestrial(ch, _) => assert_range(13, 62, ch),
            ChannelType::Catv(ch, _) => assert_range(13, 63, ch),
            ChannelType::BS(ch, id) => {
                if *ch == 7 || *ch == 17 {
                    warn!("BS-7ch and BS-17ch are ISDB-S3.");
                    false
                } else if *ch % 2 == 0 {
                    warn!("The BS channel must be an odd number.");
                    false
                } else if matches!(id, RelTsNum(num) if !(0..8).contains(num)) {
                    warn!("The relative TS number is up to 8.");
                    false
                } else {
                    assert_range(1, 23, ch)
                }
            }
            ChannelType::CS(ch, _) => {
                if *ch % 2 != 0 {
                    warn!("The CS channel must be an even number.");
                    false
                } else {
                    assert_range(2, 24, ch)
                }
            }
            ChannelType::BonChSpace(_) => true,
            _ => false,
        };

        if consistent {
            // Set
            Self {
                ch_type,
                raw_string,
            }
        } else {
            Self {
                ch_type: ChannelType::Undefined,
                raw_string,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::output::*;
    use super::representation::*;
    use super::*;

    #[test]
    fn test_isdbc_parser() {
        for (input, expected) in [
            ("ISDBC-C13", ChannelType::ISDBC(CableChannel::Catv(13), 1)),
            ("ISDBC-C63", ChannelType::ISDBC(CableChannel::Catv(63), 1)),
            ("ISDBC-1", ChannelType::ISDBC(CableChannel::VhfUhf(1), 1)),
            ("ISDBC-12", ChannelType::ISDBC(CableChannel::VhfUhf(12), 1)),
            ("ISDBC-13", ChannelType::ISDBC(CableChannel::VhfUhf(13), 1)),
            ("ISDBC-62", ChannelType::ISDBC(CableChannel::VhfUhf(62), 1)),
            ("ISDBC-C52_1", ChannelType::ISDBC(CableChannel::Catv(52), 1)),
            ("ISDBC-C52_2", ChannelType::ISDBC(CableChannel::Catv(52), 2)),
            (
                "ISDBC-C52_15",
                ChannelType::ISDBC(CableChannel::Catv(52), 15),
            ),
            (
                "ISDBC-20_3",
                ChannelType::ISDBC(CableChannel::VhfUhf(20), 3),
            ),
        ] {
            let ch = Channel::new(input, None);
            assert_eq!(ch.ch_type, expected, "{input}");
            assert_eq!(ch.get_raw_ch_name(), input);
        }
        for invalid in [
            "ISDBC-C12",
            "ISDBC-C64",
            "ISDBC-0",
            "ISDBC-63",
            "ISDBC-256",
            "ISDBC-C269",
            "ISDBC-C52_0",
            "ISDBC-C52_16",
            "ISDBC-C52_256",
            "ISDBC-C52_-1",
            "ISDBC-C52_",
            "ISDBC-C52_1extra",
            "ISDBC-C52_1_2",
            "ISDBC-C52-1",
            "ISDBC-C52extra",
            "ISDBC-",
            "ISDBC-C",
            "ISDBC-_2",
            "ISDBC-H01",
        ] {
            assert_eq!(
                Channel::new(invalid, None).ch_type,
                ChannelType::Undefined,
                "{invalid}"
            );
        }
        // C30 などの従来の CATV 指定は影響を受けない
        assert_eq!(
            Channel::new("C30", None).ch_type,
            ChannelType::Catv(30, AsIs)
        );
    }

    #[test]
    fn test_j83b_parser() {
        for (input, expected) in [
            ("J83B-H01", 93250),
            ("J83B-H02", 99750),
            ("J83B-H40", 346750),
            ("J83B-H77", 587250),
            ("J83B-159250", 159250),
            ("J83B-90000", 90000),
            ("J83B-770000", 770000),
        ] {
            let ch = Channel::new(input, None);
            assert_eq!(ch.ch_type, ChannelType::J83B(expected), "{input}");
            assert_eq!(ch.get_raw_ch_name(), input);
        }
        for invalid in [
            "J83B-H1",
            "J83B-H00",
            "J83B-H78",
            "J83B-H99",
            "J83B-H001",
            "J83B-H01_1",
            "J83B-H01_",
            "J83B-H01extra",
            "J83B-H+1",
            "J83B-H-1",
            "J83B-C40",
            "J83B-40",
            "J83B-89999",
            "J83B-770001",
            "J83B-159250_1",
            "J83B-h01",
            "J83B-H",
            "J83B-",
        ] {
            assert_eq!(
                Channel::new(invalid, None).ch_type,
                ChannelType::Undefined,
                "{invalid}"
            );
        }
    }

    #[test]
    fn test_cable_tsid_is_rejected() {
        for input in ["ISDBC-C52", "ISDBC-C52_2", "J83B-H01"] {
            assert_eq!(
                Channel::new(input, Some(17488)).ch_type,
                ChannelType::Undefined,
                "{input}"
            );
        }
    }

    #[test]
    fn test_isdbc_frequency() {
        for (physical, expected) in [
            ("1", 93000),
            ("3", 105000),
            ("C13", 111000),
            ("C21", 159000),
            ("C22", 167000),
            ("4", 173000),
            ("7", 191000),
            ("8", 195000),
            ("12", 219000),
            ("C23", 225000),
            ("C51", 393000),
            ("C52", 399000),
            ("C63", 465000),
            ("13", 473000),
            ("20", 515000),
            ("62", 767000),
        ] {
            let input = format!("ISDBC-{physical}");
            match Channel::new(&input, None).ch_type {
                ChannelType::ISDBC(ch, _) => {
                    assert_eq!(ch.frequency_khz(), expected, "{input}");
                }
                _ => panic!("Invalid channel: {input}"),
            }
        }
    }

    #[test]
    fn test_terrestrial_ch_num() {
        let ch_str = "T12";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "T13";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Terrestrial(13, TsFilter::AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "T52";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Terrestrial(52, TsFilter::AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "T62";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Terrestrial(62, TsFilter::AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "T63";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "T64";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());
    }

    #[test]
    fn test_catv_ch_num() {
        let ch_str = "C12";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "C13";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Catv(13, AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "C23";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Catv(23, AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "C63";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Catv(63, AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "C64";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());
    }

    #[test]
    fn test_bs_ch_num() {
        let ch_str = "BS0_2";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS1_2";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::BS(1, RelTsNum(2)));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS03_0";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::BS(3, RelTsNum(0)));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS4_2";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS06_0";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS07_2";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS13_3";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::BS(13, RelTsNum(3)));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS17_1";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS19_9";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "BS25_3";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());
    }

    #[test]
    fn test_cs_ch_num() {
        let ch_str = "CS0";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "CS01";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "CS2";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::CS(2, AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "CS03";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "CS04";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::CS(4, AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "CS24";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::CS(24, AsIs));
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "CS25";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());

        let ch_str = "CS26";
        let ch = Channel::new(ch_str, None);
        assert_eq!(ch.ch_type, ChannelType::Undefined);
        assert_eq!(ch.raw_string, ch_str.to_string());
    }

    #[test]
    fn test_bon_chspace_from_str() {
        let ch_str = "1-2";
        let ch = Channel::new(ch_str, None);
        assert_eq!(
            ch.ch_type,
            ChannelType::BonChSpace(ChannelSpace {
                space: 1,
                ch: 2,
                space_description: None,
                ch_description: None,
            })
        );
        assert_eq!(ch.raw_string, ch_str.to_string());
    }

    #[test]
    fn ch_to_ioctl_freq() {
        let ch_str = "T18";
        let ch = Channel::new(ch_str, None);
        let freq: IoctlFreq = ch.ch_type.into();
        assert_eq!(freq.ch, 68);
        assert_eq!(freq.slot, 0);
    }
}
