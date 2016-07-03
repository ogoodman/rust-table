extern crate libc;

use std::os::unix::io::RawFd;
use std::io;

fn num_or_oserr(n: libc::c_int) -> io::Result<libc::c_int> {
    if n < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(n)
    }
}

pub fn epoll_wait(epoll_fd: RawFd, timeout: i32, max_events: usize) -> io::Result<Vec<libc::epoll_event>>
{
    let mut buf = Vec::<libc::epoll_event>::with_capacity(max_events);

    unsafe {
        let result = libc::epoll_wait(epoll_fd, buf.as_mut_ptr(), max_events as i32, timeout);
        let num_events = try!(num_or_oserr(result)) as usize;
        buf.set_len(num_events);
    };

    Ok(buf)
}

pub fn epoll_ctl(epoll_fd: RawFd, options: i32, fd: RawFd, event: &mut libc::epoll_event) -> io::Result<()>
{
    unsafe {
        try!(num_or_oserr(libc::epoll_ctl(epoll_fd, options, fd, event as *mut libc::epoll_event)))
    };
    Ok(())
}

pub fn epoll_create(cloexec: bool) -> io::Result<RawFd> {
    let epoll_fd = unsafe {
        let fd = try!(num_or_oserr(libc::epoll_create(1)));

        if cloexec {
            let flags = try!(num_or_oserr(libc::fcntl(fd, libc::F_GETFD)));
            try!(num_or_oserr(libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC)));
        }
        fd
    };

    Ok(epoll_fd)
}

