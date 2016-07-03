use std::collections::BTreeMap;
use std::cmp::Ordering;

use util::repr;
use cmp_fi::*;

#[derive(Clone,Debug)]
pub enum JSON {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Binary(Vec<u8>),
    String(String),
    Array(Vec<JSON>),
    Object(BTreeMap<String,JSON>),
    Infinity,
}

#[derive(Debug)]
pub enum JSONParseError {
    EOF,
    UnexpectedToken(String),
}

pub fn json_encode(json: &JSON) -> String {
    match *json {
        JSON::Null => {
            String::from("null")
        },
        JSON::Bool(ref b) => {
            String::from(if *b { "true" } else { "false" })
        },
        JSON::Int(ref n) => {
            format!("{}", n)
        },
        JSON::Float(ref n) => {
            format!("{}", n)
        },
        JSON::Binary(ref b) => {
            repr(b)
        },
        JSON::String(ref s) => {
            format!("{:?}", s)
        },
        JSON::Array(ref v) => {
            let mut s = String::from("[");
            let mut first = true;
            for x in v {
                if first {
                    first = false;
                } else {
                    s.push_str(", ");
                }
                s.push_str(json_encode(x).as_str());
            }
            s.push_str("]");
            s
        },
        JSON::Object(ref m) => {
            let mut s = String::from("{");
            let mut first = true;
            for (k, v) in m {
                if first {
                    first = false;
                } else {
                    s.push_str(", ");
                }
                s.push_str(format!("{:?}", k).as_str());
                s.push_str(": ");
                s.push_str(json_encode(v).as_str());
            }
            s.push_str("}");
            s
        },
        JSON::Infinity => {
            String::from("infinity")
        },
    }
}

#[derive(Debug)]
enum Token {
    Int,
    Float,
    String,
    Punctuation,
    Identifier,
}

enum TokState {
    Begin,
    String,
    StringEscape,
    StringEnd,
    Number,
    NegNumber,
    NumberFrac,
    NumberExpSig,
    NumberExp,
    Identifier,
    Whitespace,
}

fn json_tokens<'a>(s: &'a str) -> Vec<(Token, &'a str)> {
    let mut ci = s.char_indices();
    let mut begin = 0;
    let mut state = TokState::Begin;
    let mut tokens = Vec::new();
    let mut token = Token::Punctuation;
    loop {
        match ci.next() {
            Some((i, ch)) => {
                match state {
                    TokState::Begin => {
                    },
                    TokState::NegNumber => {
                        if ch.is_digit(10) {
                            state = TokState::Number;
                            token = Token::Int;
                        } else {
                            state = TokState::Begin;
                        }
                    },
                    TokState::Number => {
                        if ch == '.' {
                            state = TokState::NumberFrac;
                            token = Token::Float;
                        } else if ch == 'e' || ch == 'E' {
                            state = TokState::NumberExpSig;
                            token = Token::Float;
                        } else if !ch.is_digit(10) {
                            state = TokState::Begin;
                        }
                    },
                    TokState::NumberFrac => {
                        if ch == 'e' || ch == 'E' {
                            state = TokState::NumberExpSig;
                        } else if !ch.is_digit(10) {
                            state = TokState::Begin;
                        }
                    },
                    TokState::NumberExpSig => {
                        if ch == '-' || ch == '+' {
                            state = TokState::NumberExp;
                        } else if ch.is_digit(10) {
                            state = TokState::NumberExp;
                        } else {
                            state = TokState::Begin;
                        }
                    },
                    TokState::NumberExp => {
                        if !ch.is_digit(10) {
                            state = TokState::Begin;
                        }
                    },
                    TokState::String => {
                        match ch {
                            '"' => {
                                state = TokState::StringEnd;
                            },
                            '\\' => {
                                state = TokState::StringEscape;
                            },
                            _ => (),
                        }
                    },
                    TokState::StringEnd => {
                        state = TokState::Begin;
                    },
                    TokState::StringEscape => {
                        state = TokState::String;
                    },
                    TokState::Identifier => {
                        if !ch.is_alphabetic() {
                            state = TokState::Begin;
                        }
                    },
                    TokState::Whitespace => {
                        if !ch.is_whitespace() {
                            begin = i;
                            state = TokState::Begin;
                        }
                    },
                };
                match state {
                    TokState::Begin => {
                        if i > begin {
                            // println!("token: {}", &s[begin..i]);
                            tokens.push((token, &s[begin..i]));
                            token = Token::Punctuation;
                            begin = i;
                        }
                        if ch == '"' {
                            state = TokState::String;
                            token = Token::String;
                        } else if ch.is_digit(10) {
                            state = TokState::Number;
                            token = Token::Int;
                        } else if ch == '-' {
                            state = TokState::NegNumber;
                        } else if ch.is_whitespace() {
                            state = TokState::Whitespace;
                        } else if ch.is_alphabetic() {
                            state = TokState::Identifier;
                            token = Token::Identifier;
                        }
                    },
                    _ => {
                    },
                }
            },
            None => {
                if begin < s.len() {
                    // println!("token: {}", &s[begin..]);
                    tokens.push((token, &s[begin..]));
                }
                break;
            },
        }
    }
    tokens
}

fn unescape_str<'a>(s: &'a str) -> String {
    let mut su = String::new();
    let n = s.len() - 1;
    for ch in s[1..n].chars() {
        if ch != '\\' {
            su.push(ch);
        }
    }
    return su;
}

fn json_object<'a, I>(mut ti: &mut I) -> Result<JSON,JSONParseError>
    where I: Iterator<Item=&'a (Token, &'a str)>
{
    // The assumption on entry is that ti just produced a '{' token.

    let mut m = BTreeMap::new();

    let mut first = true;

    loop {
        let key;

        // If not the first we need a ',' or a '}'.
        if !first {
            match ti.next() {
                Some(&(ref t, ref s)) => {
                    match *t {
                        Token::Punctuation => {
                            if *s == "}" {
                                break;
                            } else if *s != "," {
                                return Err(JSONParseError::UnexpectedToken(String::from(*s)));
                            }
                        },
                        _ => return Err(JSONParseError::UnexpectedToken(String::from(*s))),
                    }
                },
                None => return Err(JSONParseError::EOF),
            };
        }

        // The next thing is either a string or a '}'.
        match ti.next() {
            Some(&(ref t, ref s)) => {
                match *t {
                    Token::Punctuation => {
                        if *s == "}" && first {
                            break;
                        } else {
                            return Err(JSONParseError::UnexpectedToken(String::from(*s)));
                        }
                    },
                    Token::String => {
                        key = unescape_str(s);
                    },
                    _ => return Err(JSONParseError::UnexpectedToken(String::from(*s))),
                }
            },
            None => return Err(JSONParseError::EOF),
        };

        // Now we need a ':'.
        match ti.next() {
            Some(&(ref t, ref s)) => {
                match *t {
                    Token::Punctuation => {
                        if *s != ":" {
                            return Err(JSONParseError::UnexpectedToken(String::from(*s)));
                        }
                    },
                    _ => return Err(JSONParseError::UnexpectedToken(String::from(*s))),
                }
            },
            None => return Err(JSONParseError::EOF),
        };

        // And finally a JSON value.
        match json_value(ti) {
            Ok(value) => m.insert(key, value),
            Err(e) => return Err(e),
        };

        first = false;
    }

    Ok(JSON::Object(m))
}

fn json_array<'a, I>(mut ti: &mut I) -> Result<JSON,JSONParseError>
    where I: Iterator<Item=&'a (Token, &'a str)>
{
    // The assumption on entry is that ti just produced a '[' token.

    let mut v = Vec::new();

    // The next thing is either a value or a closing ']'.
    match json_value(ti) {
        Ok(val) => v.push(val),
        Err(e) => match e {
            JSONParseError::UnexpectedToken(ts) => {
                if ts == "]" {
                    return Ok(JSON::Array(v));
                } else {
                    return Err(JSONParseError::UnexpectedToken(ts));
                }
            },
            _ => return Err(e),
        },
    }

    loop {
        // We expect either ',' or ']'
        match ti.next() {
            Some(&(ref t, ref s)) => {
                match *t {
                    Token::Punctuation => {
                        if let Some(ch) = s.chars().next() {
                            match ch {
                                ']' => {
                                    break;
                                },
                                ',' => {
                                },
                                _ => return Err(JSONParseError::UnexpectedToken(String::from(*s))),
                            }
                        } else {
                            return Err(JSONParseError::UnexpectedToken(String::from(*s)));
                        }
                    },
                    _ => return Err(JSONParseError::UnexpectedToken(String::from(*s))),
                }
            },
            None => return Err(JSONParseError::EOF),
        }
        
        // We only get here if a ',' was matched above.
        match json_value(ti) {
            Ok(val) => v.push(val),
            Err(e) => return Err(e),
        }
    }

    Ok(JSON::Array(v))
}

fn json_value<'a, I>(mut ti: &mut I) -> Result<JSON,JSONParseError>
    where I: Iterator<Item=&'a (Token, &'a str)>
{
    match ti.next() {
        Some(&(ref t, ref s)) => {
            match *t {
                Token::Int => {
                    Ok(JSON::Int(s.parse::<i64>().unwrap()))
                },
                Token::Float => {
                    Ok(JSON::Float(s.parse::<f64>().unwrap()))
                },
                Token::String => {
                    Ok(JSON::String(unescape_str(s)))
                },
                Token::Identifier => {
                    if *s == "null" {
                        Ok(JSON::Null)
                    } else if *s == "true" {
                        Ok(JSON::Bool(true))
                    } else if *s == "false" {
                        Ok(JSON::Bool(false))
                    } else if *s == "infinity" {
                        Ok(JSON::Infinity)
                    } else {
                        Err(JSONParseError::UnexpectedToken(String::from(*s)))
                    }
                },
                Token::Punctuation => {
                    if let Some(ch) = s.chars().next() {
                        match ch {
                            '[' => json_array(ti),
                            '{' => json_object(ti),
                            _ => Err(JSONParseError::UnexpectedToken(String::from(*s))),
                        }
                    } else {
                        Err(JSONParseError::UnexpectedToken(String::from(*s)))
                    }
                },
            }
        },
        None => Err(JSONParseError::EOF),
    }
}

pub fn json_decode<'a>(s: &'a str) -> Result<JSON,JSONParseError> {
    let tokens = json_tokens(s);
    // println!("json_tokens: {:?}", tokens);
    json_value(&mut tokens.iter())
}

pub fn json_decode_all<'a>(s: &'a str) -> Result<Vec<JSON>,JSONParseError> {
    let tokens = json_tokens(s);
    let mut ti = tokens.iter();
    let mut v = Vec::new();
    loop {
        match json_value(&mut ti) {
            Ok(val) => v.push(val),
            Err(JSONParseError::EOF) => {
                return Ok(v);
            },
            Err(e) => {
                return Err(e);
            },
        }
    }
}

fn json_typeid(json: &JSON) -> u8 {
    match *json {
        JSON::Null => 0,
        JSON::Bool(_) => 1,
        JSON::Int(_) => 2,
        JSON::Float(_) => 2,
        JSON::Binary(_) => 3,
        JSON::String(_) => 4,
        JSON::Array(_) => 5,
        JSON::Object(_) => 6,
        JSON::Infinity => 7,
    }
}

impl PartialEq for JSON {
    fn eq(&self, other: &JSON) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for JSON {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for JSON {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JSON
{
    fn cmp(&self, other: &Self) -> Ordering {
        let self_id = json_typeid(self);
        let other_id = json_typeid(other);
        let type_cmp = self_id.cmp(&other_id);
        if type_cmp != Ordering::Equal {
            return type_cmp;
        }
        match *self {
            JSON::Null => Ordering::Equal,
            JSON::Bool(ref b) => {
                match *other {
                    JSON::Bool(ref ob) => b.cmp(ob),
                    _ => unreachable!(),
                }
            },
            JSON::Int(n) => {
                match *other {
                    JSON::Int(ref on) => n.cmp(on),
                    JSON::Float(of) => cmp_if(n, of),
                    _ => unreachable!(),
                }
            },
            JSON::Float(f) => {
                match *other {
                    JSON::Int(on) => cmp_fi(f, on),
                    JSON::Float(of) => cmp_ff(f, of),
                    _ => unreachable!(),
                }
            },
            JSON::Binary(ref v) => {
                match *other {
                    JSON::Binary(ref ov) => v.cmp(ov),
                    _ => unreachable!(),
                }
            },
            JSON::String(ref s) => {
                match *other {
                    JSON::String(ref os) => s.cmp(os),
                    _ => unreachable!(),
                }
            },
            JSON::Array(ref a) => {
                match *other {
                    JSON::Array(ref oa) => a.cmp(oa),
                    _ => unreachable!(),
                }
            },
            JSON::Object(ref bt) => {
                match *other {
                    JSON::Object(ref obt) => {
                        bt.cmp(obt)
                    },
                    _ => unreachable!(),
                }
            },
            JSON::Infinity => Ordering::Equal,
        }
    }
}

