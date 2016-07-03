//! Functions for portably serializing (resp. parsing) `f64` numbers
//! to (resp. from) their IEEE 754 bytes.

use std::f64;
use std::num::FpCategory;

fn f64_to_parts(sn: f64) -> (i8, u16, u64) {
    match sn.classify() {
        FpCategory::Nan => {
            return (1, 0x7FF, 1 << 51)
        },
        FpCategory::Infinite => {
            return (sn.signum() as i8, 0x7FF, 0)
        },
        FpCategory::Zero => {
            return (sn.signum() as i8, 0, 0)
        },
        FpCategory::Subnormal => (),
        FpCategory::Normal => (),
    }

    let sign = sn.signum() as i8;
    let n = if sign > 0 { sn } else { -sn };

    // Round exponent down (to 1) after multiplying
    // by the exponent we want 1 <= x < 2.

    let mut exp = n.log2().floor() as i32;

    // Sometimes this floating point log hasn't enough
    // precision to round down when it should.
    if n < (exp as f64).exp2() {
        exp -= 1;
    }

    // If exponent is -1023, biased exponent would be 0.
    // If exponent is -1022, biased exponent is 1.

    // An exponent of -1023 or less is represented by
    // a biased exponent of 0. We shift the number by
    // 1022 (less than the exponent) so it will have a leading 0.

    let expf;
    let bexp;
    if exp < -1022 {
        expf = -1022;
        bexp = 0u16;
    } else {
        expf = exp;
        bexp = (exp + 1023) as u16;
    };

    let normn = n / (expf as f64).exp2();

    // If the number is normal, subtract the leading 1.
    let frac = if bexp > 0 {
        normn - 1f64
    } else {
        normn
    };

    let mant = ((52f64).exp2() * frac) as u64;

    (sign, bexp, mant)
}

fn f64_from_parts(sign: i8, bexp: u16, mant: u64) -> f64 {
    if bexp == 0x7FF {
        if mant > 0 {
            return f64::NAN;
        }
        return if sign > 0 { f64::INFINITY } else { f64::NEG_INFINITY };
    }
    if bexp == 0 && mant == 0 {
        return if sign > 0 { 0f64 } else { -0f64 };
    }

    let frac = mant as f64 / (52f64).exp2();
    let normn = if bexp > 0 { frac + 1f64 } else { frac };
    let expf = if bexp > 0 { bexp as i32 - 1023 } else { -1022 };

    normn * (expf as f64).exp2() * sign as f64
}

/*
// NOTE: this is not used, except for checking visually that our
// conversions agree with the underlying IEEE-754 representation,
// assuming that's what the platform uses. The whole point of the
// rest of the code is to do it all in a platform-independent way.

fn pmem(n: f64) {
    let buf: [u8; 8] = unsafe { std::mem::transmute(n) };
    print!("repr: ");
    for i in 0..8 {
        print!("{:02X}", buf[7-i]);
    }
    print!("\n");
}

fn pvec(v: &Vec<u8>) {
    print!("bvec: ");
    for i in 0..8 {
        print!("{:02X}", v[i]);
    }
    print!("\n");
}

fn analyse(n: f64) {
    // pmem(n);

    let (sign, bexp, mant) = f64_to_parts(n);

    println!("be/m: {:03X}{:013X}", if sign < 0 { bexp + 0x800 } else { bexp }, mant);

    let v = f64_to_bytes(n);
    pvec(&v);

    let nn = f64_from_parts(sign, bexp, mant);

    if n.is_nan() {
        println!("both nan: {}", nn.is_nan());
    } else {
        println!("equal: {}", n == nn);
    }

    let nb = f64_from_bytes(v.as_slice());
    if n.is_nan() {
        println!("both nan: {}", nb.is_nan());
    } else {
        println!("equal: {}", n == nb);
    }
}
*/

/// Convert a 64-bit floating-point number to its IEEE 754 bytes.
pub fn f64_to_bytes(n: f64) -> Vec<u8> {
    let (sign, bexp, mut mant) = f64_to_parts(n);
    let mut v = vec![0u8; 8];

    v[0] = (if sign < 0 { bexp + 0x800 } else { bexp } >> 4) as u8;
    v[1] = (bexp << 4 & 0xFF) as u8 + (mant >> 48) as u8;

    for i in 0..6 {
        v[7 - i] = (mant & 0xFF) as u8;
        mant >>= 8;
    }
    v
}

/// Parse the IEEE 754 bytes of a 64-bit floating-point number.
///
/// NOTE: this will panic if `v` isn't at least 8 bytes long.
pub fn f64_from_bytes(v: &[u8]) -> f64 {
    let sign: i8 = if v[0] & 0x80 > 0 { -1 } else { 1 };
    let bexp: u16 = (((v[0] & 0x7F) as u16) << 4) + ((v[1] as u16) >> 4);
    let mut mant = (v[1] & 0xF) as u64;
    for i in 2..8 {
        mant <<= 8;
        mant += v[i] as u64;
    }
    f64_from_parts(sign, bexp, mant)
}

/// Errors that can occur when decoding a hex encoded string
#[cfg(test)]
#[derive(Copy, Clone, Debug)]
enum FromHexError {
    /// The input contained a character not part of the hex format
    InvalidHexCharacter(char, usize),
    /// The input had an invalid length
    InvalidHexLength,
}

#[cfg(test)]
fn from_hex(s: &str) -> Result<Vec<u8>, FromHexError> {
    let mut b = Vec::with_capacity(s.len() / 2);
    let mut modulus = 0;
    let mut buf = 0;

    for (idx, byte) in s.bytes().enumerate() {
        buf <<= 4;

        match byte {
            b'A'...b'F' => buf |= byte - b'A' + 10,
            b'a'...b'f' => buf |= byte - b'a' + 10,
            b'0'...b'9' => buf |= byte - b'0',
            _ => {
                let ch = s[idx..].chars().next().unwrap();
                return Err(FromHexError::InvalidHexCharacter(ch, idx))
            }
        }

        modulus += 1;
        if modulus == 2 {
            modulus = 0;
            b.push(buf);
        }
    }

    match modulus {
        0 => Ok(b.into_iter().collect()),
        _ => Err(FromHexError::InvalidHexLength),
    }
}

#[test]
pub fn check_f64_conversions() {
    let pi: f64 = 3.14159265;
    let pi_vec = from_hex("400921FB53C8D4F1").unwrap();

    assert_eq!(f64_to_bytes(pi), pi_vec);
    assert_eq!(f64_from_bytes(&pi_vec), pi);

    let mpi_vec = from_hex("C00921FB53C8D4F1").unwrap();

    assert_eq!(f64_to_bytes(-pi), mpi_vec);
    assert_eq!(f64_from_bytes(&mpi_vec), -pi);

    let edge: f64 = 1.125 * (-1022f64).exp2();
    let edge_vec = from_hex("0012000000000000").unwrap();

    assert_eq!(f64_to_bytes(edge), edge_vec);
    assert_eq!(f64_from_bytes(&edge_vec), edge);

    let sub: f64 = 1.125 * (-1023f64).exp2();
    let sub_vec = from_hex("0009000000000000").unwrap();

    assert_eq!(f64_to_bytes(sub), sub_vec);
    assert_eq!(f64_from_bytes(&sub_vec), sub);

    let nan_vec = from_hex("7FF8000000000000").unwrap();

    assert_eq!(f64_to_bytes(f64::NAN), nan_vec);
    assert!(f64_from_bytes(&nan_vec).is_nan());

    let inf_vec = from_hex("7FF0000000000000").unwrap();

    assert_eq!(f64_to_bytes(f64::INFINITY), inf_vec);
    assert_eq!(f64_from_bytes(&inf_vec), f64::INFINITY);

    let neginf_vec = from_hex("FFF0000000000000").unwrap();

    assert_eq!(f64_to_bytes(f64::NEG_INFINITY), neginf_vec);
    assert_eq!(f64_from_bytes(&neginf_vec), f64::NEG_INFINITY);

    let max_vec = from_hex("7FEFFFFFFFFFFFFF").unwrap();

    assert_eq!(f64_to_bytes(f64::MAX), max_vec);
    assert_eq!(f64_from_bytes(&max_vec), f64::MAX);

    let min_vec = from_hex("0010000000000000").unwrap();

    assert_eq!(f64_to_bytes(f64::MIN_POSITIVE), min_vec);
    assert_eq!(f64_from_bytes(&min_vec), f64::MIN_POSITIVE);

    let eps_vec = from_hex("3CB0000000000000").unwrap();

    assert_eq!(f64_to_bytes(f64::EPSILON), eps_vec);
    assert_eq!(f64_from_bytes(&eps_vec), f64::EPSILON);

    let zero_vec = from_hex("0000000000000000").unwrap();

    assert_eq!(f64_to_bytes(0f64), zero_vec);
    assert_eq!(f64_from_bytes(&zero_vec), 0f64);

    let mzero_vec = from_hex("8000000000000000").unwrap();

    assert_eq!(f64_to_bytes(-0f64), mzero_vec);
    assert_eq!(f64_from_bytes(&mzero_vec), -0f64);
}
