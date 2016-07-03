extern crate table;
extern crate readline;

use std::env;
use std::io;

use table::table::Table;
use table::util::repr;
use table::encode::Encode;
use table::decode::Decode;
use table::record::print_json;
use table::json::*;

use readline::{readline, add_history};

trait MyIO
    where Self: Sized
{
    fn parse(&String) -> Result<Self,String>;
    fn repr(&self) -> String;
}

impl MyIO for Vec<u8> {
    fn parse(s: &String) -> Result<Self,String> {
        Ok(s.as_bytes().to_vec())
    }
    fn repr(&self) -> String {
        repr(self)
    }
}

impl MyIO for i64 {
    fn parse(s: &String) -> Result<Self,String> {
        match s.parse::<Self>() {
            Err(e) => Err(format!("\"{}\" is not a number: {:?}", s, e)),
            Ok(n) => Ok(n),
        }
    }
    fn repr(&self) -> String {
        format!("{}", self)
    }
}

type Bytes = Vec<u8>;

struct JSONTableIter<'a> {
    raw: std::collections::btree_map::Iter<'a, i64, Vec<u8>>,
}

type JSONRow = std::collections::BTreeMap<String, JSON>;

impl<'a> Iterator for JSONTableIter<'a> {
    type Item = (i64, JSONRow);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.raw.next() {
                Some((k, ref v)) => {
                    match JSON::decode(&mut io::Cursor::new(v)) {
                        Ok(JSON::Object(row)) => return Some((*k, row)),
                        _ => (),
                    }
                },
                None => return None,
            }
        }
    }
}

struct JSONTable<'a> {
    raw: &'a mut Table<i64, Vec<u8>>,
}

impl<'a> IntoIterator for &'a JSONTable<'a> {
    type Item = (i64, JSONRow);
    type IntoIter = JSONTableIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        JSONTableIter { raw: (&*self.raw).into_iter() }
    }
}


fn tdemo<K: MyIO + Ord + Encode + Decode>(path: &str, args: &[String])
{
    if args.len() == 2 {
        if args[1] == "items" {
            let t: Table<K,Vec<u8>> = match Table::open(path) {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            for (key, value) in &t {
                println!("{}: {}", key.repr(), value.repr());
            }
        } else if args[1] == "compact" {
            let mut t: Table<K,Vec<u8>> = match Table::open_rw(path) {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            match t.compact(path) {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(()) => (),
            };
        }
    } else if args.len() == 3 {
        if args[1] == "get" {
            let key = match K::parse(&args[2]) {
                Err(e) => { println!("arg 2 {}", e); return; },
                Ok(k) => k,
            };
            let t: Table<K,Vec<u8>> = match Table::open(path) {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            match t.get(&key) {
                Some(v) => println!("value: {:?}", v),
                None => println!("no value for key: {}", key.repr()),
            };
        } else if args[1] == "remove" {
            let key = match K::parse(&args[2]) {
                Err(e) => { println!("arg 2 {}", e); return; },
                Ok(k) => k,
            };
            let mut t: Table<K,Vec<u8>> = match Table::open_rw(path) {
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
            let key = match K::parse(&args[2]) {
                Err(e) => { println!("arg 2 {}", e); return; },
                Ok(k) => k,
            };
            let mut t: Table<K,Vec<u8>> = match Table::open_rw(path) {
                Err(e) => { println!("error opening table : {:?}", e); return; },
                Ok(t) => t,
            };
            let value = args[3].as_bytes().to_vec();
            match t.insert(key, value) {
                Err(e) => { println!("error inserting: {:?}", e); },
                Ok(_) => { println!("table updated."); },
            };
        }
    }
}

fn unwrap_vs(data: &JSON) -> Option<Vec<String>> {
    match *data {
        JSON::Array(ref vj) => {
            let mut vs = Vec::new();
            for j in vj {
                match *j {
                    JSON::String(ref s) => {
                        vs.push(s.clone());
                    },
                    _ => return None,
                }
            }
            Some(vs)
        },
        _ => None,
    }
}

fn table_cmd(rt: &mut Table<i64,Vec<u8>>, cmd: &str, data: &JSON) {
    let mut t = JSONTable { raw: rt };
    if cmd == "insert" {
        let key = match t.raw.max_key() {
            Some(ref k) => *k + 1,
            None => 0,
        };
        let mut value: Vec<u8> = Vec::new();
        data.encode(&mut value).unwrap();
        t.raw.insert(key, value).unwrap();
    } else if cmd == "index" {
        if let Some(keys) = unwrap_vs(data) {
            for (pk, row) in &t {
                let mut vj = Vec::new();
                for key in &keys {
                    vj.push(
                        if key == "#" {
                            JSON::Int(pk)
                        } else {
                            match row.get(key) {
                                Some(val) => val.clone(),
                                None => JSON::Null,
                            }
                        }
                    )
                };
                println!("{}", json_encode(&JSON::Array(vj)));
            }
        } else {
            println!("index requires an array of key strings");
        }
    } else if cmd == "select" {
        for (key, mut m) in &t {
            m.insert(String::from("#"), JSON::Int(key));
            println!("{}", json_encode(&JSON::Object(m)));
        }
    } else {
        println!("cmd: {} {:?}", cmd, data);
    }
}

fn json_table_cmd(t: &mut Table<i64,JSONRow>, cmd: &str, data: &JSON) {
    if cmd == "insert" {
        let row = match *data {
            JSON::Object(ref m) => m,
            _ => {
                println!("can only insert JSON Objects");
                return;
            },
        };
        let key = match t.max_key() {
            Some(ref k) => *k + 1,
            None => 0,
        };
        t.insert(key, row.clone()).unwrap();
    } else if cmd == "index" {
        if let Some(keys) = unwrap_vs(data) {
            for (pk, row) in &*t {
                let mut vj = Vec::new();
                for key in &keys {
                    vj.push(
                        if key == "#" {
                            JSON::Int(*pk)
                        } else {
                            match row.get(key) {
                                Some(val) => val.clone(),
                                None => JSON::Null,
                            }
                        }
                    )
                };
                println!("{}", json_encode(&JSON::Array(vj)));
            }
        } else {
            println!("index requires an array of key strings");
        }
    } else if cmd == "select" {
        for (key, m) in &*t {
            let mut mc = m.clone();
            mc.insert(String::from("#"), JSON::Int(*key));
            println!("{}", json_encode(&JSON::Object(mc)));
        }
    } else {
        println!("cmd: {} {:?}", cmd, data);
    }
}

fn repl() {
    let mut t: Table<i64,Vec<u8>> = match Table::open_rw("baz.bt") {
        Err(e) => { println!("error opening table : {:?}", e); return; },
        Ok(t) => t,
    };
    loop {
        match readline("> ") {
            Ok(l) => {
                add_history(l.as_str()).unwrap();

                let mut cmd_args = l.splitn(2, ' ');
                let cmd;
                if let Some(c) = cmd_args.next() {
                    cmd = c;
                } else {
                    println!("no command given!");
                    continue;
                }
                if let Some(a) = cmd_args.next() {
                    if cmd == "cmp" {
                        match json_decode_all(a) {
                            Ok(vals) => {
                                if vals.len() != 2 {
                                    println!("need 2 values to compare");
                                } else {
                                    println!("{:?} {:?} {:?}",  vals[0], vals[0].cmp(&vals[1]), vals[1]);
                                }
                            },
                            Err(e) => {
                                println!("parse error: {:?}", e);
                            },
                        }
                    } else {
                        match json_decode(a) {
                            Ok(data) => {
                                table_cmd(&mut t, cmd, &data);
                            },
                            Err(_) => {
                                println!("cmd: {} --invalid-json--", cmd);
                            },
                        }
                    }
                } else {
                    table_cmd(&mut t, cmd, &JSON::Null);
                }
            },
            Err(_) => {
                println!("");
                break;
            },
        }
    }
}

fn json_repl() {
    let mut t: Table<i64,JSONRow> = match Table::open_rw("ben.bt") {
        Err(e) => { println!("error opening table : {:?}", e); return; },
        Ok(t) => t,
    };
    loop {
        match readline("> ") {
            Ok(l) => {
                add_history(l.as_str()).unwrap();

                let mut cmd_args = l.splitn(2, ' ');
                let cmd;
                if let Some(c) = cmd_args.next() {
                    cmd = c;
                } else {
                    println!("no command given!");
                    continue;
                }
                if let Some(a) = cmd_args.next() {
                    if cmd == "cmp" {
                        match json_decode_all(a) {
                            Ok(vals) => {
                                if vals.len() != 2 {
                                    println!("need 2 values to compare");
                                } else {
                                    println!("{:?} {:?} {:?}",  vals[0], vals[0].cmp(&vals[1]), vals[1]);
                                }
                            },
                            Err(e) => {
                                println!("parse error: {:?}", e);
                            },
                        }
                    } else {
                        match json_decode(a) {
                            Ok(data) => {
                                json_table_cmd(&mut t, cmd, &data);
                            },
                            Err(_) => {
                                println!("cmd: {} --invalid-json--", cmd);
                            },
                        }
                    }
                } else {
                    json_table_cmd(&mut t, cmd, &JSON::Null);
                }
            },
            Err(_) => {
                println!("");
                break;
            },
        }
    }
}

fn main() {
    let mut args = Vec::new();
    args.extend(env::args());

    if args.len() < 2 {
        println!("Usage: table <tbl>|demo <cmd> <args>..");
        return;
    }

    if args[1] == "json" {
        print_json();
    } else if args[1] == "repl" {
        repl();
    } else if args[1] == "json_repl" {
        json_repl();
    } else if args[1] == "is" {
        tdemo::<i64>("foo.bt", &args[1..]);
    } else if args[1] == "ss" {
        tdemo::<Bytes>("bar.bt", &args[1..]);
    }
}
