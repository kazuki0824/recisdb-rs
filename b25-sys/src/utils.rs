extern crate byteorder;

use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian};
use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::access_control::types::{EmmExtendedKeys, EmmReceivingKeyPair, WorkingKey};

mod cryptography00;
mod cryptography40;

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
        let (aligned, bottom) = {
            let mut buf = vec![self.iv];
            //store u64
            let count = input.len() / 8;
            let mut pos = Cursor::new(input);
            for _x in 0..count {
                buf.push(pos.read_u64::<BigEndian>().unwrap());
            }
            let leftover_pos = pos.position() as usize;
            (buf, pos.into_inner().split_off(leftover_pos))
            //(buf, pos.remaining_slice())
        };

        //CBC
        let key = if id % 2 == 0 { self.key.0 } else { self.key.1 };
        let coded: Vec<u64> = aligned
            .iter()
            .map(|x| cryptography00::crypto_block00(*x, key, false))
            .collect();

        //re-store u8, and emit the bottom of the aligned part
        let (mut out, last) = {
            let mut last_aligned_part = 0;
            let mut c = Cursor::new(Vec::<u8>::new());
            for x in 1..aligned.len() {
                //Chain
                last_aligned_part = aligned[x];
                let xor = aligned[x - 1] ^ coded.get(x).unwrap();
                c.write_u64::<BigEndian>(xor).unwrap();
            }
            (c.into_inner(), last_aligned_part)
        };

        //resolve unaligned stubs
        let tail = unsafe {
            let tail = cryptography00::crypto_block00(last, key, true);
            std::mem::transmute::<u64, [u8; 8]>(tail)
        };
        for x in 0..bottom.len() {
            out.push(bottom[x] ^ tail[7 - x])
        }
        out
    }
}

pub(crate) struct BlockConversionSolver40 {
    pub ex_key: EmmExtendedKeys,
    iv: u64,
}
impl BlockConversionSolver40 {
    pub(crate) fn new(key: EmmReceivingKeyPair) -> Self {
        Self {
            ex_key: EmmExtendedKeys {
                0: key.card_id,
                1: cryptography40::key_schedule40(key.key),
            },
            iv: 0x11096919991927fe,
        }
    }
    pub(crate) fn try_convert(&self, input: Vec<u8>, protocol: u8) -> Option<Vec<u8>> {
        let (aligned, bottom) = {
            let mut buf = vec![self.iv];
            //store u64
            let count = input.len() / 8;
            let mut pos = Cursor::new(input);
            for _x in 0..count {
                buf.push(pos.read_u64::<LittleEndian>().unwrap());
            }
            let leftover_pos = pos.position() as usize;
            (buf, pos.into_inner().split_off(leftover_pos))
            //(buf, pos.remaining_slice())
        };

        //CBC
        let key = self.ex_key.1;
        let coded: Vec<u64> = aligned
            .iter()
            .map(|x| cryptography40::crypto_block_40(*x, key, false, protocol))
            .collect();

        //re-store u8, and emit the bottom of the aligned part
        let (mut out, last) = {
            let mut last_aligned_part = 0;
            let mut c = Cursor::new(Vec::<u8>::new());
            for x in 1..aligned.len() {
                //Chain
                last_aligned_part = aligned[x];
                let xor = aligned[x - 1] ^ coded.get(x).unwrap();
                c.write_u64::<LittleEndian>(xor).unwrap();
            }
            (c.into_inner(), last_aligned_part)
        };

        //resolve unaligned stubs
        let tail = unsafe {
            let tail = cryptography40::crypto_block_40(last, key, true, protocol);
            std::mem::transmute::<u64, [u8; 8]>(tail)
        };
        for x in 0..bottom.len() {
            out.push(bottom[x] ^ tail[7 - x])
        }
        Some(out)
        //TODO: 1 Implement tests, 2 Generate MAC to check it
    }
}

#[cfg(test)]
mod tests {
    use crate::access_control::types::WorkingKey;
    use crate::utils::BlockConversionSolver00;

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
