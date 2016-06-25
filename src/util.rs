use std::io::Write;

pub fn repr<T: AsRef<[u8]>>(v: T) -> String {
    let hex = b"0123456789ABCDEF";
    let mut b: Vec<u8> = Vec::new();
    b.write(b"\"").unwrap();
    let mut buf = [b"\\"[0], 0u8, 0u8];
    for c in v.as_ref() {
        if *c < 32 || *c > 126 {
            buf[1] = hex[(*c >> 4) as usize];
            buf[2] = hex[(*c & 0xF) as usize];
            b.write(&buf[..]).unwrap();
        } else {
            buf[1] = *c;
            b.write(&buf[1..2]).unwrap();
        }
    }
    b.write(b"\"").unwrap();
    String::from_utf8(b).unwrap()
}

