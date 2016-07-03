extern crate table;

use std::io::{Read, Cursor};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{RawFd, AsRawFd};

use std::time;

use table::reactor;
use table::reactor::{Reactor, ReactorAction, Reader, Task, Scheduler};
use table::decode::{Decode, DecodeError, DecodeStats};

#[derive(Clone, Copy)]
enum ReaderState {
    Begin,
    ReadSize(usize),
}

struct SimpleReader {
    stream: TcpStream,
    buffer: Vec<u8>,
    state: ReaderState,
}

struct SimpleListener {
    listener: TcpListener,
}

impl AsRawFd for SimpleReader {
    fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}

impl Task for SimpleReader {
    fn run(&mut self) -> Vec<ReactorAction> {
        let mut actions = Vec::new();
        let mut buf = [0u8; 1024];
        match self.stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    actions.push(ReactorAction::Remove(self.stream.as_raw_fd()));
                    return actions;
                }
                self.buffer.extend(&buf[..n]);
            }
            Err(e) => {
                println!("error {:?} reading", e);
                actions.push(ReactorAction::Remove(self.stream.as_raw_fd()));
                return actions;
            }
        }

        loop {
            let mut consumed: usize = 0;
            match self.state {
                ReaderState::Begin => {
                    let mut s = DecodeStats { read: 0, discarded: 0 };
                    let mut c = Cursor::new(&self.buffer);
                    match u64::decode_stats(&mut c, &mut s) {
                        Ok(n) => {
                            self.state = ReaderState::ReadSize(n as usize);
                            consumed = s.read;
                        }
                        Err(DecodeError::EOF) => (), // Ok, just need more data.
                        Err(_) => {
                            println!("protocol error!");
                        }
                    }
                }
                ReaderState::ReadSize(msg_len) => {
                    if self.buffer.len() >= msg_len {
                        // let msg = self.buffer;
                        // self.buffer = msg.split_off(msg_len);
                        let text = String::from_utf8_lossy(&self.buffer[..msg_len]);
                        println!("got: {}", text);
                        consumed = msg_len;
                        self.state = ReaderState::Begin;
                    }
                },
            };
            if consumed == 0 {
                break;
            }
            self.buffer = self.buffer.split_off(consumed);
        }

        actions
    }
}

impl Reader for SimpleReader {}

impl AsRawFd for SimpleListener {
    fn as_raw_fd(&self) -> RawFd {
        self.listener.as_raw_fd()
    }
}

impl Task for SimpleListener {
    fn run(&mut self) -> Vec<ReactorAction> {
        let mut actions = Vec::new();
        match self.listener.accept() {
            Ok((stream, addr)) => {
                println!("connection from {:?}", addr);
                actions.push(ReactorAction::Add(Box::new(SimpleReader { stream: stream, buffer: Vec::new(), state: ReaderState::Begin })));
            }
            Err(e) => {
                println!("error in accept: {:?}", e);
                actions.push(ReactorAction::Remove(self.listener.as_raw_fd()));
            }
        }
        actions
    }
}

impl Reader for SimpleListener {}

struct Heartbeat {
    next: time::Instant
}

impl Task for Heartbeat {
    fn run(&mut self) -> Vec<ReactorAction> {
        println!("heartbeat");
        self.next = time::Instant::now() + time::Duration::new(10,0);
        Vec::new()
    }
}

impl Scheduler for Heartbeat {
    fn due(&self) -> i32 {
        let now = time::Instant::now();
        if now > self.next {
            0
        } else {
            let pause = self.next - now;
            let pause_ms = pause.as_secs() * 1000 + (pause.subsec_nanos() / 1000000) as u64;
            if pause_ms > (i32::max_value() as u64) { i32::max_value() } else { pause_ms as i32 }
        }
    }
}

fn main() {

    let scheduler = Heartbeat{ next: time::Instant::now() + time::Duration::new(10,0) };
    let mut reactor = reactor::EpollReactor::new_with_scheduler(Box::new(scheduler)).unwrap();

    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();

    reactor.add_reader(Box::new(SimpleListener { listener: listener }));
    reactor.run();

}
