use std::io;
use std::collections::BTreeMap;

use encode::Encode;

#[cfg(test)]
use encode::encode;

#[derive(Debug)]
pub enum DecodeError {
    IOError { err: io::Error },
    EOF,
    Null,
    PartialRead,
}

#[derive(Default)]
pub struct DecodeStats {
    read: usize,
    discarded: usize,
}

impl DecodeStats {
    pub fn read(&self) -> usize { self.read }
    pub fn discarded(&self) -> usize { self.discarded }
}

pub trait Decode : Sized {
    fn decode<T: io::Read>(src: &mut T) -> Result<Self, DecodeError> {
        let mut stats = DecodeStats { read: 0, discarded: 0 };
        Self::decode_stats(src, &mut stats)
    }

    fn decode_stats<T: io::Read>(&mut T, &mut DecodeStats) -> Result<Self, DecodeError>;
}

impl Decode for u64 {
    fn decode_stats<T: io::Read>(src: &mut T, stats: &mut DecodeStats) ->
        Result<Self, DecodeError>
    {
        let mut buff = [0u8; 1];

        match src.read(&mut buff) {
            Ok(nread) => if nread == 0 { return Err(DecodeError::EOF); },
            Err(err) => return Err(DecodeError::IOError { err: err }),
        };
        stats.read += 1;

        if buff[0] < 0xFD {
            Ok(buff[0] as u64)
        } else if buff[0] == 0xFD {
            let mut b2 = [0u8; 2];
            match src.read(&mut b2) {
                Ok(nread) => {
                    stats.read += nread;
                    if nread < 2 {
                        stats.discarded += nread;
                        return Err(DecodeError::PartialRead);
                    }
                },
                Err(err) => return Err(DecodeError::IOError { err: err }),
            };
            Ok(0x100 * (b2[0] as u64) + b2[1] as u64)
        } else if buff[0] == 0xFE {
            let mut b8 = [0u8; 8];
            match src.read(&mut b8) {
                Ok(nread) => {
                    stats.read += nread;
                    if nread < 8 {
                        stats.discarded += nread;
                        return Err(DecodeError::PartialRead);
                    }
                }
                Err(err) => return Err(DecodeError::IOError { err: err }),
            };
            let mut n: u64 = 0;
            for i in 0..8 {
                n <<= 8;
                n += b8[i] as u64;
            }
            Ok(n)
        } else {
            Err(DecodeError::Null)
        }
    }
}

impl Decode for i64 {
    fn decode_stats<T: io::Read>(src: &mut T, stats: &mut DecodeStats) ->
        Result<Self, DecodeError>
    {
        let mut buff = [0u8; 1];
        match src.read(&mut buff) {
            Ok(nread) => if nread == 0 { return Err(DecodeError::EOF); },
            Err(err) => return Err(DecodeError::IOError { err: err }),
        };
        stats.read += 1;

        if (buff[0] as i8) > -0x7F {
            Ok(buff[0] as i8 as i64)
        } else if buff[0] == 0x81 {
            let mut b2 = [0u8; 2];
            match src.read(&mut b2) {
                Ok(nread) => {
                    stats.read += nread;
                    if nread < 2 {
                        stats.discarded += nread;
                        return Err(DecodeError::PartialRead);
                    }
                },
                Err(err) => return Err(DecodeError::IOError { err: err }),
            };
            Ok((0x100 * (b2[0] as i8 as i16) + b2[1] as i16) as i64)
        } else {
            let mut b8 = [0u8; 8];
            match src.read(&mut b8) {
                Ok(nread) => {
                    stats.read += nread;
                    if nread < 8 {
                        stats.discarded += nread;
                        return Err(DecodeError::PartialRead);
                    }
                },
                Err(err) => return Err(DecodeError::IOError { err: err }),
            };
            let mut n: i64 = b8[0] as i8 as i64;
            for i in 1..8 {
                n <<= 8;
                n += b8[i] as i64;
            }
            Ok(n)
        }
    }
}

impl Decode for Vec<u8> {
    fn decode_stats<T: io::Read>(src: &mut T, stats: &mut DecodeStats) ->
        Result<Self, DecodeError>
    {
        let pos = stats.read;
        let n = match u64::decode_stats(src, stats) {
            Err(err) => return Err(err),
            Ok(n) => n,
        } as usize;
        let mut v = vec![0u8; n];
        match src.read(&mut v[..]) {
            Err(err) => Err(DecodeError::IOError { err: err }),
            Ok(nr) => {
                stats.read += nr;
                if nr == n {
                    Ok(v)
                } else {
                    stats.discarded += stats.read - pos;
                    Err(DecodeError::PartialRead)
                }
            },
        }
    }
}

type Bytes = Vec<u8>;
type DataMap = BTreeMap<i64, Bytes>;

impl Decode for DataMap {
    fn decode_stats<T: io::Read>(src: &mut T, stats: &mut DecodeStats) ->
        Result<Self, DecodeError>
    {
        let mut data: DataMap = BTreeMap::new();
        loop {
            let pos = stats.read;
            let key = match i64::decode_stats(src, stats) {
                Err(err) => match err {
                    DecodeError::EOF => return Ok(data),
                    _ => return Err(err),
                },
                Ok(key) => key,
            };
            let keysize = stats.read - pos;
            match Bytes::decode_stats(src, stats) {
                Err(err) => match err {
                    DecodeError::Null => {
                        // the discard iteslf is wasted space.
                        stats.discarded += keysize + 1;
                        match data.remove(&key) {
                            Some(value) => {
                                // original insert and this remove are now redundant
                                stats.discarded += keysize + value.encode_size();
                            },
                            None => (),
                        }
                    },
                    _ => return Err(err),
                },
                Ok(value) => {
                    match data.insert(key, value) {
                        Some(value) => {
                            // original insert including key are now redundant
                            stats.discarded += keysize + value.encode_size();
                        },
                        None => (),
                    }
                },
            }
        }
    }
}

#[cfg(test)]
fn round_trip<T: Encode + Decode>(val: &T) -> Option<T> {
    let mut s = io::Cursor::new(encode(val));
    match T::decode(&mut s) {
        Err(_) => None,
        Ok(v) => Some(v),
    }
}

#[test]
fn test_decoders() {
    let mut src = io::Cursor::new(vec![0x2Au8]);
    assert_eq!(u64::decode(&mut src).unwrap(), 42u64);

    assert_eq!(round_trip(&320u64), Some(320u64));
    assert_eq!(round_trip(&123456789u64), Some(123456789u64));

    let mut data: Vec<u8> = Vec::new();
    data.extend(b"Hello");
    assert_eq!(round_trip(&data), Some(data));

    assert_eq!(round_trip(&10i64), Some(10i64));
    assert_eq!(round_trip(&-10i64), Some(-10i64));
    assert_eq!(round_trip(&320i64), Some(320i64));
    assert_eq!(round_trip(&-320i64), Some(-320i64));
    assert_eq!(round_trip(&123456789i64), Some(123456789i64));
    assert_eq!(round_trip(&-123456789i64), Some(-123456789i64));
    assert_eq!(round_trip(&-0x7Fi64), Some(-0x7Fi64));
    assert_eq!(round_trip(&-0x80i64), Some(-0x80i64));

}
