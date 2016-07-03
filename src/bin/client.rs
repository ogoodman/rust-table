extern crate table;

use std::io::Write;
use std::env;
use std::net::TcpStream;

use table::encode::Encode;

fn main() {
    let mut args = Vec::new();
    args.extend(env::args());

    match TcpStream::connect("127.0.0.1:8000") {
        Ok(mut stream) => {
            for a in &args[1..] {
                let n = a.len() as u64;
                n.encode(&mut stream).unwrap();
                let _ = stream.write(a.as_bytes());
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
