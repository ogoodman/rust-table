extern crate table;

use std::env;

use table::demo::encodings_demo;
use table::table::Table;
use table::util::repr;

fn main() {
    let mut args = Vec::new();
    args.extend(env::args());

    if args.len() == 2 {
        if args[1] == "items" {
            let t = match Table::open("foo.bt") {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            for (key, value) in &t {
                println!("{}: {}", key, repr(value));
            }
        } else if args[1] == "demo" {
            encodings_demo();
        } else if args[1] == "compact" {
            let mut t = match Table::open_rw("foo.bt") {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            match t.compact("foo.bt") {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(()) => (),
            };
        }
    } else if args.len() == 3 {
        if args[1] == "get" {
            let key = match args[2].parse::<i64>() {
                Err(e) => { println!("arg 2 must be a number : {:?}", e); return; },
                Ok(n) => n,
            };
            let t = match Table::open("foo.bt") {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            match t.get(&key) {
                Some(v) => println!("value: {:?}", v),
                None => println!("no value for key: {}", key),
            };
        } else if args[1] == "remove" {
            let key = match args[2].parse::<i64>() {
                Err(e) => { println!("arg 2 must be a number : {:?}", e); return; },
                Ok(n) => n,
            };
            let mut t = match Table::open_rw("foo.bt") {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            match t.remove(&key) {
                Err(e) => { println!("error removing: {:?}", e); },
                Ok(_) => { println!("table updated."); },
            };
        }
    } else if args.len() == 4 {
        if args[1] == "set" {
            let key = match args[2].parse::<i64>() {
                Err(e) => { println!("arg 2 must be a number : {:?}", e); return; },
                Ok(n) => n,
            };
            let mut t = match Table::open_rw("foo.bt") {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            match t.insert(key, args.pop().unwrap().into_bytes()) {
                Err(e) => { println!("error inserting: {:?}", e); },
                Ok(_) => { println!("table updated."); },
            };
        }
    }
}
