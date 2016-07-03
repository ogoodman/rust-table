extern crate table;

use std::io;
use std::collections::BTreeMap;

use table::encode::{ Encode, encode, encode_to_hex };
use table::decode::*;

fn main() {
    type DataMap = BTreeMap<i64,Vec<u8>>;

    println!("42 encoded: {}", encode_to_hex(&42u64));
    println!("320 encoded: {}", encode_to_hex(&320u64));
    println!("123456789 encoded: {}", encode_to_hex(&123456789u64));
    println!("NULL encoded: {}", encode_to_hex(&None));
    let data: &[u8] = b"Hello";
    println!("Hello encoded: {}", encode_to_hex(&data));

    let mut src = io::Cursor::new(vec![0x2Au8]);
    println!("0x2A decoded: {}", u64::decode(&mut src).unwrap());

    let mut s2 = io::Cursor::new(encode(&320u64));
    println!("320 decoded: {}", u64::decode(&mut s2).unwrap());

    let mut s3 = io::Cursor::new(encode(&123456789u64));
    println!("123456789 decoded: {}", u64::decode(&mut s3).unwrap());

    type Bytes = Vec<u8>;

    let mut s4 = io::Cursor::new(encode(&data));
    println!("Hello decoded: {:?}", Bytes::decode(&mut s4).unwrap());

    let mut s5: Box<io::Read> = Box::new(io::Cursor::new(Vec::new() as Vec<u8>));
    let mut buf = [0u8; 1];
    println!("Empty read: {:?}", s5.read(&mut buf));

    let mut s6 = io::Cursor::new(Vec::new() as Vec<u8>);
    println!("Decode empty: {:?}", Bytes::decode(&mut s6));

    let mut s7 = io::Cursor::new(encode(&10i64));
    println!("10 decoded: {}", i64::decode(&mut s7).unwrap());
    let mut s8 = io::Cursor::new(encode(&-10i64));
    println!("-10 decoded: {}", i64::decode(&mut s8).unwrap());

    let mut s9 = io::Cursor::new(encode(&320i64));
    println!("320 decoded: {}", i64::decode(&mut s9).unwrap());
    let mut s10 = io::Cursor::new(encode(&-320i64));
    println!("-320 decoded: {}", i64::decode(&mut s10).unwrap());

    let mut s11 = io::Cursor::new(encode(&123456789i64));
    println!("123456789 decoded: {}", i64::decode(&mut s11).unwrap());
    let mut s12 = io::Cursor::new(encode(&-123456789i64));
    println!("-123456789 decoded: {}", i64::decode(&mut s12).unwrap());

    let mut s13 = io::Cursor::new(encode(&-0x7Fi64));
    println!("-0x7F decoded: {}", i64::decode(&mut s13).unwrap());
    let mut s14 = io::Cursor::new(encode(&-0x80i64));
    println!("-0x80 decoded: {}", i64::decode(&mut s14).unwrap());

    let mut s15 = io::Cursor::new(vec![0x81u8, 0u8]);
    println!("incomplete decoded: {:?}", i64::decode(&mut s15));
    let mut s16 = io::Cursor::new(vec![0x80u8, 0u8, 0u8, 0u8]);
    println!("incomplete decoded: {:?}", i64::decode(&mut s16));

    let mut data = BTreeMap::new();
    let v: Vec<u8> = From::from(&b"Tom"[..]);
    data.insert(5i64, v);
    data.insert(17i64, From::from(&b"Dick"[..]));
    println!("data.get(5): {:?}", data.get(&5).unwrap());

    for (key, value) in &data {
        println!("{}: {:?}", key, value);
    }
    println!("data encoded: {}", encode_to_hex(&data));


    let mut v2: Vec<u8> = Vec::new();
    data.encode(&mut v2).unwrap();

    let d2;
    {
        let mut s = io::Cursor::new(&mut v2);
        d2 = DataMap::decode(&mut s).unwrap();
    }
    println!("data.len(): {}", d2.len());

    (&17i64).encode(&mut v2).unwrap();
    None.encode(&mut v2).unwrap();

    let d2;
    {
        let mut s = io::Cursor::new(&mut v2);
        d2 = DataMap::decode(&mut s).unwrap();
    }
    println!("data.len(): {}", d2.len());
}

