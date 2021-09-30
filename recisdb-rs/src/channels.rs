use fancy_regex::Regex;

#[repr(C)]
pub struct Freq {
    pub ch: i32,
    pub slot: i32,
}

#[derive(Clone, Copy)]
pub enum ChannelType {
    Terrestrial,
    CATV,
    BS,
    CS,
    Undefined,
}
#[derive(Clone)]
pub struct Channel {
    pub ch_type: ChannelType,
    pub raw_string: String,
    pub physical_ch_num: u8,
    pub stream_id: i32,
}

impl Channel {
    pub fn from_ch_str(ch_str: impl Into<String>) -> Channel {
        let ch_str = ch_str.into();

        let isdb_t_regex = Regex::new(r"(?<=[TC])\d{1,2}\b").unwrap();
        let cs_regex = Regex::new(r"(?<=CS)\d?[02468]\b").unwrap();
        let bs_regex = Regex::new(r"(?<=BS)\d[13579]_[01234567]\b").unwrap();

        if let Ok(Some(m)) = isdb_t_regex.find(&ch_str) {
            let first_letter = ch_str.chars().nth(0).unwrap();
            let ch_type = if first_letter == 'T' {
                ChannelType::Terrestrial
            } else {
                ChannelType::CATV
            };
            let physical_ch_num = m.as_str().parse().unwrap();

            Channel {
                ch_type,
                raw_string: ch_str.clone(),
                physical_ch_num,
                stream_id: 0,
            }
        } else if cs_regex.is_match(&ch_str).unwrap() {
            let ch_type = ChannelType::CS;
            let caps = cs_regex.captures(&ch_str).unwrap().unwrap();
            let result_str = caps.get(0).map_or("", |m| m.as_str());
            let physical_ch_num = result_str.parse().unwrap();

            Channel {
                ch_type,
                raw_string: ch_str.clone(),
                physical_ch_num,
                stream_id: 0,
            }
        } else if bs_regex.is_match(&ch_str).unwrap() {
            let ch_type = ChannelType::BS;
            let caps = cs_regex.captures(&ch_str).unwrap().unwrap();
            let result_str = caps.get(0).map_or("", |m| m.as_str());

            let underline_loc = result_str.rfind('_').unwrap();

            let physical_ch_num = (result_str[0..underline_loc - 1]).parse().unwrap();
            let stream_id: i32 = result_str[underline_loc + 1..underline_loc + 1]
                .parse()
                .unwrap();

            Channel {
                ch_type,
                raw_string: ch_str.clone(),
                physical_ch_num,
                stream_id,
            }
        } else {
            Channel {
                ch_type: ChannelType::Undefined,
                raw_string: ch_str,
                physical_ch_num: 0,
                stream_id: 0,
            }
        }
    }
    pub fn to_freq(&self, freq_offset: i32) -> Freq {
        let ch_num = self.physical_ch_num;
        let ioctl_channel = match self.ch_type {
            ChannelType::Terrestrial if (ch_num >= 13) && (ch_num <= 52) => ch_num + 50,
            ChannelType::CATV if (ch_num >= 23) && (ch_num <= 63) => ch_num - 1,
            ChannelType::CATV if (ch_num >= 13) && (ch_num <= 22) => ch_num - 10,
            ChannelType::CS if (ch_num >= 2) && (ch_num <= 24) && (ch_num % 2 == 0) => {
                ch_num / 2 + 11
            }
            ChannelType::BS if (ch_num >= 1) && (ch_num <= 23) && (ch_num % 2 == 1) => ch_num / 2,
            ChannelType::Undefined => unimplemented!(),
            _ => panic!("Invalid channel."),
        };
        let slot = match self.ch_type {
            ChannelType::CS => 0,
            ChannelType::BS => self.stream_id,
            _ => freq_offset,
        };
        Freq {
            ch: ioctl_channel as i32,
            slot,
        }
    }
    pub fn get_dvb_freq(&self) -> i32 {
        -1
    }
}
