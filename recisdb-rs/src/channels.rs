use crate::channels::representation::TsFilter::{AbsTsId, AsIs, RelTsNum};
pub(crate) use crate::channels::representation::{ChannelSpace, ChannelType};
use log::{error, warn};

pub mod output {
    use crate::channels::representation::{ChannelType, TsFilter};

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

                ChannelType::Bon(_) => unimplemented!(),
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

            let hz = match &value {
                ChannelType::Terrestrial(..) | ChannelType::Catv(..) => {
                    if (freq.ch >= 3 && freq.ch < 12) || (freq.ch >= 22 && freq.ch <= 62) {
                        /* CATV C13-C22ch, C23-C63ch */
                        93143 + freq.ch * 6000 + freq.slot /* addfreq */
                    } else if freq.ch == 12 {
                        93143 + freq.ch * 6000 + freq.slot
                    } else if freq.ch >= 63 && freq.ch <= 112 {
                        /* UHF 13-62ch */
                        95143 + freq.ch * 6000 + freq.slot /* addfreq */
                    } else {
                        unreachable!()
                    }
                }
                ChannelType::BS(..) | ChannelType::CS(..) => {
                    if freq.ch < 0 {
                        unreachable!()
                    } else if freq.ch < 12 {
                        /* BS */
                        1049480 + (38360 * freq.ch)
                    } else if freq.ch < 24 {
                        /* CS */
                        1613000 + (40000 * (freq.ch - 12))
                    } else {
                        unreachable!()
                    }
                }
                _ => unreachable!(),
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
                freq_hz: hz as u32,
                stream_id,
            }
        }
    }
}

mod representation {
    use std::fmt::Display;

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
        Bon(ChannelSpace),
        Undefined,
    }

    impl Display for ChannelType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            type T = ChannelType;
            match self {
                T::Terrestrial(ch, TsFilter::AsIs) => write!(f, "GR: {ch}"),
                T::Catv(ch, TsFilter::AsIs) => write!(f, "CATV: {ch}"),
                T::Terrestrial(ch, tsid) => write!(f, "GR: {} (TS Filter={:?})", ch, tsid),
                T::Catv(ch, tsid) => write!(f, "CATV: {} (TS Filter={:?})", ch, tsid),
                T::BS(ch, tsid) => write!(f, "BS: {}, {:?}", ch, tsid),
                T::CS(ch, tsid) => write!(f, "CS: {}, {:?}", ch, tsid),
                T::Bon(ChannelSpace { ch, space, .. }) => {
                    write!(f, "BonDriver: Ch={ch}, Space={space}")
                }
                _ => write!(f, "Undefined"),
            }
        }
    }
}

mod parser {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::u32;
    use nom::sequence::separated_pair;
    use nom::IResult;

    pub(crate) fn get_result(input: &str) -> IResult<&str, &str> {
        alt((tag("BS"), tag("CS"), tag("T"), tag("C")))(input)
    }

    pub(crate) fn parse_integer_pair(input: &str) -> IResult<&str, (u32, u32)> {
        separated_pair(u32, alt((tag("-"), tag("_"))), u32)(input)
    }
}

pub struct Channel {
    pub ch_type: ChannelType,
    raw_string: String,
}

impl Channel {
    pub fn get_raw_ch_name(&self) -> &str {
        self.raw_string.as_str()
    }

    pub fn new(ch_str: impl Into<String>, override_stream_id: Option<u32>) -> Self {
        let raw_string = ch_str.into();

        let ch_type = if let Ok(val) = raw_string.parse::<u8>() {
            ChannelType::Terrestrial(val, AsIs)
        } else {
            // Parse
            match parser::get_result(&raw_string) {
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
                    Ok((_, (first, second))) => ChannelType::Bon(ChannelSpace {
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
            ChannelType::Bon(_) => true,
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
            ChannelType::Bon(ChannelSpace {
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
