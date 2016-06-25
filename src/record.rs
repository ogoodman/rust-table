use std::collections::BTreeMap;
use std::io::Write;

#[derive(Debug)]
enum JSON {
    Null,
    Bool(bool),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<JSON>),
    Object(BTreeMap<String,JSON>),
    Number(f64),
}



pub fn rdemo() {
    println!("A null: {:?}", JSON::Null);
    println!("A bool: {:?}", JSON::Bool(true));
    println!("A string: {:?}", JSON::String(String::from("Fred")));
}
