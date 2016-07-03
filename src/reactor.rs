extern crate libc;

use std::os::unix::io::{RawFd, AsRawFd};
use std::collections::BTreeMap;
use std::io;
use std::ops::DerefMut;

use epoll::*;

// We need a BTreeMap<i64, Box<Task>> of tasks.
// We also need an index of when those tasks become due.
// A Task and its Schedule could be completely separate.
// Task addition and removal independent of schedule/unschedule.
// Scheduling operations cause an index of next-due to be updated.

// All of that can be implemented independently. All the Reactor
// needs to provide is a way to get a next-due time for a single
// Task. 

pub trait Task {
    fn run(&mut self) -> Vec<ReactorAction>;
}

pub trait Reader : AsRawFd + Task {
}

pub trait Scheduler : Task {
    // Returns how many ms to wait before calling run. -1 means forever.
    fn due(&self) -> i32;
}

pub trait Reactor {
    fn run(&mut self);
    fn stop(&mut self);
    fn add_reader(&mut self, Box<Reader>);
    fn remove_reader(&mut self, RawFd);
}

pub struct EpollReactor {
    readers: BTreeMap<RawFd,Box<Reader>>,
    run: bool,
    epoll_fd: RawFd,
    scheduler: Box<Scheduler>,
}

pub enum ReactorAction {
    Add(Box<Reader>),
    Remove(RawFd),
    Stop,
}

fn do_todos(reactor: &mut Reactor, todo: Vec<ReactorAction>) {
    for action in todo {
        match action {
            ReactorAction::Add(r) => reactor.add_reader(r),
            ReactorAction::Remove(fd) => reactor.remove_reader(fd),
            ReactorAction::Stop => reactor.stop(),
        }
    }
}

impl Reactor for EpollReactor {
    fn run(&mut self) {
        loop {
            let mut todo = Vec::new();

            let timeout = self.scheduler.due();

            match epoll_wait(self.epoll_fd, timeout, 1) {
                Err(e) => {
                    println!("unexpected error in epoll_wait() {:?}", e);
                    break;
                }
                Ok(events) => {
                    for ev in events {
                        let fd = ev.u64 as RawFd;
                        let opt_r = self.readers.get_mut(&fd);
                        if let Some(mut reader) = opt_r {
                            todo.extend(reader.deref_mut().run());
                        }
                    }
                    if self.scheduler.due() == 0 {
                        todo.extend(self.scheduler.run());
                    }
                }
            };
            do_todos(self, todo);
            if !self.run || self.readers.len() == 0 {
                break;
            }
        }
    }

    fn stop(&mut self) {
        self.run = false;
    }

    fn add_reader(&mut self, reader: Box<Reader>) {
        let fd = reader.as_raw_fd();
        self.readers.insert(fd, reader);

        let mut ev = libc::epoll_event { events: libc::EPOLLIN as u32, u64: fd as u64 };
        epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut ev).unwrap();
    }

    fn remove_reader(&mut self, fd: RawFd) {
        self.readers.remove(&fd);
    }
}

struct NullScheduler {
}

impl Task for NullScheduler {
    fn run(&mut self) -> Vec<ReactorAction>
    {
        Vec::new()
    }
}

impl Scheduler for NullScheduler {
    fn due(&self) -> i32
    {
        -1
    }
}

impl EpollReactor {
    pub fn new() -> io::Result<EpollReactor> {
        Self::new_with_scheduler(Box::new(NullScheduler{}))
    }

    pub fn new_with_scheduler(s: Box<Scheduler>) -> io::Result<EpollReactor> {
        match epoll_create(false) {
            Ok(fd) => Ok(
                EpollReactor {
                    readers: BTreeMap::new(),
                    run: true,
                    epoll_fd: fd,
                    scheduler: s,
                }
            ),
            Err(e) => Err(e)
        }
    }
}


