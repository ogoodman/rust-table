use std::io::Write;
use std::io;
use std::collections::BTreeMap;

pub trait Encode {
    fn encode<T: Write>(&self, &mut T) -> io::Result<()>;
    fn encode_size(&self) -> usize;
}

impl Encode for u64 {
    fn encode<T: Write>(&self, out: &mut T) -> io::Result<()> {
        let mut buff = [0u8; 9];
        let n = *self;
        return if n < 0xFD {
            buff[0] = (n & 0xFF) as u8;
            out.write_all(&buff[..1])
        } else if n < 0x10000 {
            buff[0] = 0xFD;
            buff[1] = ((n >> 8) & 0xFF) as u8;
            buff[2] = (n & 0xFF) as u8; 
            out.write_all(&buff[..3])
        } else {
            buff[0] = 0xFE;
            let mut nn = n;
            for i in 0..8 {
                buff[8 - i] = (nn & 0xFF) as u8;
                nn >>= 8;
            }
            out.write_all(&buff[..9])
        }
    }

    fn encode_size(&self) -> usize {
        let n = *self;
        if n < 0xFD { 1 } else if n < 0x10000 { 3 } else { 9 }
    }
}

// Signed, not null: 0x81 = i16, 0x80 = i64.

impl Encode for i64 {
    fn encode<T: Write>(&self, out: &mut T) -> io::Result<()> {
        let mut buff = [0u8; 9];
        let n = *self;
        return if -0x7F < n && n < 0x80 {
            buff[0] = (n & 0xFF) as u8;
            out.write_all(&buff[..1])
        } else if -0x8000 <= n && n < 0x8000 {
            buff[0] = 0x81;
            buff[1] = ((n >> 8) & 0xFF) as u8;
            buff[2] = (n & 0xFF) as u8; 
            out.write_all(&buff[..3])
        } else {
            buff[0] = 0x80;
            let mut nn = n;
            for i in 0..8 {
                buff[8 - i] = (nn & 0xFF) as u8;
                nn >>= 8;
            }
            out.write_all(&buff[..9])
        }
    }

    fn encode_size(&self) -> usize {
        let n = *self;
        if -0x7F < n && n < 0x80 { 1 } else if -0x8000 <= n && n < 0x8000 { 3 } else { 9 }
    }
}

impl Encode for Option<u64> {
    fn encode<T: Write>(&self, out: &mut T) -> io::Result<()> {
        let buff = [0xFFu8; 1];
        match *self {
            None => out.write_all(&buff),
            Some(n) => n.encode(out),
        }
    }

    fn encode_size(&self) -> usize {
        match *self {
            None => 1,
            Some(n) => n.encode_size(),
        }
    }
}

impl<'a> Encode for &'a [u8] {
    fn encode<T: Write>(&self, out: &mut T) -> io::Result<()> {
        let n = self.len() as u64;
        match n.encode(out) {
            Result::Ok(()) => out.write_all(self),
            Result::Err(err) => Result::Err(err),
        }
    }

    fn encode_size(&self) -> usize {
        (self.len() as u64).encode_size() + self.len()
    }
}

impl Encode for Vec<u8> {
    fn encode<T: Write>(&self, out: &mut T) -> io::Result<()> {
        self.as_slice().encode(out)
    }

    fn encode_size(&self) -> usize {
        self.as_slice().encode_size()
    }
}

impl Encode for BTreeMap<i64, Vec<u8> > {
    fn encode<T: Write>(&self, out: &mut T) -> io::Result<()> {
        for (key, value) in self {
            match key.encode(out) {
                Err(err) => return Err(err),
                Ok(()) => (),
            };
            match value.as_slice().encode(out) {
                Err(err) => return Err(err),
                Ok(()) => (),
            };
        }
        Ok(())
    }

    fn encode_size(&self) -> usize {
        let mut n: usize = 0;
        for (key, value) in self {
            n += key.encode_size();
            n += value.encode_size();
        }
        n
    }
}

pub fn encode<T: Encode>(ob: &T) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    ob.encode(&mut v).unwrap();
    v
}

pub fn encode_to_hex<T: Encode>(ob: &T) -> String {
    let mut v: Vec<u8> = Vec::new();

    ob.encode(&mut v).unwrap();

    let mut s: Vec<u8> = Vec::new();
    for b in v.iter() {
        write!(s, "{:02X}", b).unwrap();
    }

    return String::from_utf8(s).unwrap();
}

#[test]
fn test_encode() {
    assert_eq!(encode_to_hex(&42u64), "2A");
    assert_eq!(encode_to_hex(&320u64), "FD0140");
    assert_eq!(encode_to_hex(&123456789u64), "FE00000000075BCD15");
    assert_eq!(encode_to_hex(&None), "FF");
    let data: &[u8] = b"Hello";
    assert_eq!(encode_to_hex(&data), "0548656C6C6F");

    assert_eq!(encode_to_hex(&10i64), "0A");
    assert_eq!(encode_to_hex(&-10i64), "F6");
    assert_eq!(encode_to_hex(&320i64), "810140");
    assert_eq!(encode_to_hex(&-320i64), "81FEC0");
    assert_eq!(encode_to_hex(&123456789i64), "8000000000075BCD15");
    assert_eq!(encode_to_hex(&-123456789i64), "80FFFFFFFFF8A432EB");

    assert_eq!(encode_to_hex(&-0x7Fi64), "81FF81");
    assert_eq!(encode_to_hex(&-0x80i64), "81FF80");
}
