//! This module contains the main abstraction, which is a
//! thin layer over epoll.
#![allow(dead_code, unused)]

use std::{
    io::{self, Result},
    net::TcpStream,
    os::fd::AsRawFd,
};

use crate::ffi;

type Events = Vec<ffi::Event>;

/// Represents the event queue itself.
///
pub struct Poll {
    // a Registry is specific to an event queue / Poll instance
    registry: Registry,
}

impl Poll {
    /// Create a new event queue
    pub fn new() -> Result<Self> {
        let res = unsafe { ffi::epoll_create(1) };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            registry: Registry { raw_fd: res },
        })
    }

    /// return reference to the registry
    ///
    /// This reference can be used to register interest
    /// to be notified of new events.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Blocks the thread it's called on until an event is ready
    /// or it times out, whichever occurs first.
    pub fn poll(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        let epfd = self.registry.raw_fd;

        // a timeout of -1 means block indefinitely
        let timeout = timeout.unwrap_or(-1);
        let max_events = events.capacity() as i32;

        // block on epoll_wait
        let res = unsafe { ffi::epoll_wait(epfd, events.as_mut_ptr(), max_events, timeout) };

        // we would get a res of 0 if a timeout occurs before an event has happened
        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        // On notification, `events` should be populated with at most max_events
        // so we must set the length of `events`
        unsafe { events.set_len(res as usize) };
        Ok(())
    }
}

/// A handle that allows us to register interest in new events
pub struct Registry {
    raw_fd: i32, // 4 bytes
}

impl Registry {
    /// interests: indicates what kind of event we are interested in
    pub fn register<T>(&self, source: &T, token: usize, interests: i32) -> Result<()>
    where
        T: AsRawFd,
    {
        // create a new event (dropped at end of this method)
        let mut event = ffi::Event {
            events: interests as u32, // bitmask for events we are interested in
            epoll_data: token,
        };

        // only use the `add` flag
        let op = ffi::EPOLL_CTL_ADD;
        ffi::print_event_debug(&event);
        ffi::check(event.events as i32);

        let res = unsafe { ffi::epoll_ctl(self.raw_fd, op, source.as_raw_fd(), &mut event) };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for Registry {
    /// Close the epoll file descriptor
    fn drop(&mut self) {
        let res = unsafe { ffi::close(self.raw_fd) };

        if res < 0 {
            let err = io::Error::last_os_error();
            eprintln!("error closing epoll file descriptor: {err:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_send<T: Send>() {}
    fn test_sync<T: Sync>() {}

    #[test]
    fn test_marker_traits() {
        test_send::<Registry>();
        test_sync::<Registry>();

        test_send::<Poll>();
        test_sync::<Poll>();
    }
}
