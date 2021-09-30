extern crate byteorder;

use std::io::Cursor;

use byteorder::BigEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};
use crate::access_control::types::WorkingKey;

mod cryptography00;


pub(crate) struct BlockConversionSolver00 {
    key: ([u32; 4], [u32; 4]),
    iv: u64,
}

impl BlockConversionSolver00 {
    pub(crate) fn new(k: WorkingKey, protocol: u8) -> Self {
        Self {
            key: (
                cryptography00::key_schedule00(k.0, protocol),
                cryptography00::key_schedule00(k.1, protocol),
            ),
            iv: 0xfe27199919690911,
        }
    }
    pub(crate) fn convert(&self, input: Vec<u8>, id: u8) -> Vec<u8> {
        let mut buf = vec![self.iv];

        //store u64
        let count = input.len() / 8;
        let mut pos = Cursor::new(input);

        let mut last_aligned_part: u64 = 0;
        for _x in 0..count {
            last_aligned_part = pos.read_u64::<BigEndian>().unwrap();
            buf.push(last_aligned_part);
        }
        let bottom = &pos.into_inner()[8 * count..];
        //let bottom = pos.remaining_slice();

        //CBC
        let key = if id % 2 == 0 { self.key.0 } else { self.key.1 };
        let coded: Vec<u64> = buf
            .iter()
            .map(|x| cryptography00::crypto_block00(*x, key, false))
            .collect();
        for x in 1..buf.len() {
            buf[x - 1] ^= coded.get(x).unwrap();
        }

        //re-store u8
        let out = Vec::<u8>::new();
        let mut c = Cursor::new(out);
        let count = buf.len() - 1;
        for x in 0..count {
            c.write_u64::<BigEndian>(buf[x]).unwrap();
        }

        //resolve unaligned stubs
        let tail = unsafe {
            let tail = cryptography00::crypto_block00(last_aligned_part, key, true);
            std::mem::transmute::<u64, [u8; 8]>(tail)
        };
        let mut out = c.into_inner();
        for x in 0..bottom.len() {
            out.push(bottom[x] ^ tail[7 - x])
        }
        out
    }
}


pub(crate) struct BlockConversionSolver40
{

}

#[cfg(test)]
mod tests {
    use crate::utils::BlockConversionSolver00;
    use crate::access_control::types::WorkingKey;

    #[test]
    fn cbc() {
        let p = [
            114, 116, 0, 226, 60, 55, 128, 37, 162, 232, 216, 253, 93, 130, 66, 98, 1, 207, 13, 1,
            1, 1, 1, 22, 239, 206, 247 as u8,
        ];

        let c = [
            159, 58, 82, 76, 87, 23, 65, 15, 177, 90, 205, 103, 37, 55, 180, 88, 150, 134, 168,
            201, 187, 210, 81, 67, 87, 190, 140,
        ];
        let b = BlockConversionSolver00::new(WorkingKey::DEFAULT, 0);
        let c = b.convert(c.into(), 1);
        assert!(p.eq(&c[0..27]));
    }
}
