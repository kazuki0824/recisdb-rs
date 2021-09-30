fn parity(x: u32) -> u32 {
    let mut y = x;
    y ^= x >> 16;
    y ^= y >> 8;
    y ^= y >> 4;
    y ^= y >> 2;
    y ^= y >> 1;
    return y & 1;
}

fn round_function_00(x: u32, key: u32, flavor: u8) -> u32 {
    let flag_salt_choice = (flavor & 2) == 0b10;
    let flag_stirring_method = (flavor & 1) == 0b1;

    let salt: u32 = if flag_salt_choice { 0x5353 } else { 0 };
    let x_left = x >> 16;
    let k_left = key >> 16;
    let x_right = x & 0xffff;
    let k_right = key & 0xffff;

    let x_left = x_left + k_left + salt;
    let x_right = x_right + k_right + salt;
    let x = (x_left << 16) | (x_right & 0xffff);

    // half-byte mixture
    let x = ((x & 0xf0f0f0f0) >> 4) | ((x & 0x0f0f0f0f) << 4);
    // circular(barrel) shift
    let k = (key << 1) | (key >> 31);

    let x = if parity(x & k) != 0 { x ^ (!k) } else { x };
    let x = if flag_stirring_method {
        (x & 0xaa55aa55) | ((x & 0x55005500) >> 7) | ((x & 0x00aa00aa) << 7)
    } else {
        (x & 0x55aa55aa) | ((x & 0xaa00aa00) >> 9) | ((x & 0x00550055) << 9)
    };
    let x = (x & 0x00ffff00) | (x >> 24) | (x << 24);
    let x = x ^ ((x << 24) | (x >> 8)) ^ ((x << 25) | (x >> 7));

    return x;
}

pub fn key_schedule00(key: u64, protocol: u8) -> [u32; 4] {
    let mut kext: [u32; 4] = [0, 0, 0, 0];
    kext[0] = (key >> 32) as u32;
    kext[1] = key as u32;
    kext[2] = 0x08090a0b;
    kext[3] = 0x0c0d0e0f;

    let mut chain: u32 = if (protocol & 0x0c) != 0 {
        0x84e5c4e7
    } else {
        0x6aa32b6f
    };
    for i in 0..8 {
        chain = round_function_00(kext[i & 3], chain, 0);
        kext[i & 3] = chain;
    }
    return kext;
}

pub fn crypto_block00(cipher: u64, extended_key: [u32; 4], encrypt: bool) -> u64 {
    let flavor = [1, 0, 1, 2, 2, 2, 0, 2, 1, 3, 0, 2, 1, 0, 0, 1];

    let mut left: u32 = (cipher >> 32) as u32;
    let mut right = cipher as u32;
    if encrypt {
        let mut r = 0;
        loop {
            left ^= round_function_00(right, extended_key[r & 3], flavor[r]);
            r += 1;
            right ^= round_function_00(left, extended_key[r & 3], flavor[r]);
            if r == 15 {
                break;
            };
            r += 1;
        }
    } else {
        let mut r = 15;
        loop {
            left ^= round_function_00(right, extended_key[r & 3], flavor[r]);
            r -= 1;
            right ^= round_function_00(left, extended_key[r & 3], flavor[r]);
            if r == 0 {
                break;
            };
            r -= 1;
        }
    }

    return ((right as u64) << 32) | (left as u64);
}

#[cfg(test)]
mod tests {
    use crate::utils::cryptography00::{crypto_block00, key_schedule00, parity, round_function_00};

    #[test]
    fn calc_parity1() {
        let x: u32 = 0xF0F0F0F0;
        assert_eq!(parity(x), 0);
    }
    #[test]
    fn calc_parity2() {
        let x: u32 = 0x38B80801;
        assert_eq!(parity(x), 1);
    }
    #[test]
    fn block() {
        let key: u64 = 0;
        let kext = key_schedule00(key, 0);
        let p = crypto_block00(0, kext, false);
        for x in kext.iter() {
            println!("{}", x);
        }
        println!("{}", p);
        assert_eq!(p, 16467544269716193282);
    }
    #[test]
    fn round1() {
        let p = round_function_00(1010, 0, 0);
        println!("{}", p);
        assert_eq!(p, 978197038);
    }
    #[test]
    fn round2() {
        let p = round_function_00(99999999, 99999999, 1);
        println!("{}", p);
        assert_eq!(p, 1922248979);
    }
}
