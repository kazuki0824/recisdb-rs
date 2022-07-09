use cryptography_00::{feistel, round_function};
use tail_cbc::cipher;
use tail_cbc::cipher::{BlockCipher, InvalidLength, Key, KeyInit, KeySizeUser};
use tail_cbc::cipher::typenum::{U16, U3, U8};

type KeySize00 = U16;
pub(crate) type BlockSize00 = U8;
type ParBlocksSize00 = U3;

pub(crate) fn expand_00(key: u64, protocol: u8) -> Key<Block00> {
    let mut exp00: [u32; 4] = [0, 0, 0, 0];
    exp00[0] = (key >> 32) as u32;
    exp00[1] = key as u32;
    exp00[2] = 0x08090a0b;
    exp00[3] = 0x0c0d0e0f;

    let mut chain: u32 = if (protocol & 0x0c) != 0 {
        0x84e5c4e7
    } else {
        0x6aa32b6f
    };
    for i in 0..8 {
        chain = round_function(exp00[i & 3], chain, 0);
        exp00[i & 3] = chain;
    }
    Key::<Block00>::from_exact_iter(
        //no vec
        exp00.into_iter().map(|x| x.to_le_bytes()).flatten(),
    )
        .unwrap()
}

#[derive(Clone)]
pub struct Block00 {
    keys: [u32; 4],
}
impl BlockCipher for Block00 {}
impl KeySizeUser for Block00 {
    type KeySize = KeySize00;
}
impl KeyInit for Block00 {
    fn new(key: &Key<Self>) -> Self {
        Self::new_from_slice(key).unwrap()
    }
    fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
        match key.len() {
            16 => {
                let mut keys = [0; 4];
                keys[0] = u32::from_le_bytes(key[0..4].try_into().unwrap());
                keys[1] = u32::from_le_bytes(key[4..8].try_into().unwrap());
                keys[2] = u32::from_le_bytes(key[8..12].try_into().unwrap());
                keys[3] = u32::from_le_bytes(key[12..16].try_into().unwrap());
                Ok(Self { keys })
            }
            _ => Err(InvalidLength),
        }
    }
}

crate::impl_block_encdec!(
    Block00, BlockSize00, ParBlocksSize00, cipher, block,
    encrypt: {
        let b = block.get_in();
        let mut b: u64 = u64::from_be_bytes(b[0..8].try_into().unwrap());
        for i in 0..8
        {
            if i % 2 == 0
            {
                b = feistel(b, cipher.keys[0], cipher.keys[1], ROUND_INDEX[i * 2], ROUND_INDEX[i * 2 + 1]);
            }
            else
            {
                b = feistel(b, cipher.keys[2], cipher.keys[3], ROUND_INDEX[i * 2], ROUND_INDEX[i * 2 + 1]);
            }
        }
        //reverse the order of the two 32-bit words
        b = (b << 32) | (b >> 32);
        let block = block.get_out();
        block[0..8].copy_from_slice(&b.to_be_bytes());
    }
    decrypt: {
        let b = block.get_in();
        let mut b: u64 = u64::from_be_bytes(b[0..8].try_into().unwrap());
        for i in (0..=7_usize).rev()
        {
            if i % 2 == 1
            {
                b = feistel(b, cipher.keys[3], cipher.keys[2], ROUND_INDEX[i * 2 + 1], ROUND_INDEX[i * 2]);
            }
            else
            {
                b = feistel(b, cipher.keys[1], cipher.keys[0], ROUND_INDEX[i * 2 + 1], ROUND_INDEX[i * 2]);
            }
        }
        //reverse the order of the two 32-bit words
        b = (b << 32) | (b >> 32);
        let block = block.get_out();
        block[0..8].copy_from_slice(&b.to_be_bytes());
    }
);
const ROUND_INDEX: [u8; 16] = [1, 0, 1, 2, 2, 2, 0, 2, 1, 3, 0, 2, 1, 0, 0, 1];


#[test]
fn cbc() {
    use cipher::KeyIvInit;
    use tail_cbc::UnalignedBytesDecryptMut;
    type Block00CbcDec = tail_cbc::Decryptor<Block00>;

    let p = [
        114, 116, 0, 226, 60, 55, 128, 37, 162, 232, 216, 253, 93, 130, 66, 98, 1, 207, 13, 1, 1,
        1, 1, 22, 239, 206, 247 as u8,
    ];

    let c = [
        159, 58, 82, 76, 87, 23, 65, 15, 177, 90, 205, 103, 37, 55, 180, 88, 150, 134, 168, 201,
        187, 210, 81, 67, 87, 190, 140 as u8,
    ];
    let mut b = Block00CbcDec::new(
        &expand_00(0x15f8c5bf840b6694u64, 0),
        &0xfe27199919690911u64.to_be_bytes().into(),
    );

    let mut result = c.clone();
    b.decrypt_bytes_b2b_mut(&c, &mut result).expect("TODO: panic message");

    assert_eq!(result, p);
}

#[test]
fn test_zero_dec_block00() {
    use cipher::{Block, BlockDecrypt, KeyInit};

    let key: u64 = 0;
    let key = expand_00(key, 0);
    let decryptor = Block00::new(&key);
    let mut c: Block<Block00> = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 as u8].into();
    assert_eq!(
        c.clone().to_vec(),
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    decryptor.decrypt_block(&mut c);

    let p = u64::from_be_bytes(c.into());
    println!("{}", p);
    assert_eq!(p, 16467544269716193282u64);
}

#[test]
fn test_zero_enc_block00() {
    use cipher::{Block, BlockEncrypt, KeyInit};

    let key: u64 = 0x15f8c5bf840b6694u64;
    let key = expand_00(key, 0);
    let decryptor = Block00::new(&key);
    let mut p: Block<Block00> = 10846542336961433923u64.to_be_bytes().into();

    decryptor.encrypt_block(&mut p);

    let c = u64::from_be_bytes(p.into());
    assert_eq!(c, 13290258443935467468u64);
}