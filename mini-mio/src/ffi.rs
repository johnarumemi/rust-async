//! This module contains the code related to the syscals we
//! need to communicate with the host operating system.
//!
//! # FIXES:
//! The number is identical to the number in the GitHub issue tracker
//!
//! ## FIX ISSUE #5
//! See: https://github.com/PacktPublishing/Asynchronous-Programming-in-Rust/issues/5
//! Readers reported wrong results when running the example on ARM64 instruction set
//! (aarch64). The reason turned out to be that the `Event` struct is only `repr(packed)`
//! on `x86-64` systems due to backwards compatibility. Fixed by conditionally
//! compiling the #[repr(packed)] attribute.

#![allow(dead_code, unused)]

// repr(packed) forces Rust to strip any padding, and only align the type to a byte
// note: The OS syscall expects the struct we use to be packed.
#[derive(Debug)]
#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(packed))] // only use packed on x86_64
pub struct Event {
    // bitmask of events that we are interested in
    pub(crate) events: u32,

    // token to identify event source
    pub(crate) epoll_data: usize,
}

impl Event {
    pub fn token(&self) -> usize {
        self.epoll_data
    }
}
// ------------------------------------------------------------
// System calls
// ------------------------------------------------------------

pub const EPOLL_CTL_ADD: i32 = 1; // opcode for adding to epoll event queue

// bitflags for events we are interested in
pub const EPOLLIN: i32 = 0x1; // read operations on the file handle
pub const EPOLLET: i32 = 1 << 31; // edge-triggered mode

#[cfg(target_os = "linux")]
#[link(name = "c")] // link to C standard library / libc
extern "C" {

    /// open an epoll file descriptor
    ///
    /// https://man7.org/linux/man-pages/man2/epoll_create.2.html
    ///
    /// #include <sys/epoll.h>
    ///
    /// int epoll_create(int size);
    ///
    /// The argument is there only for historical reasons and it will be ignored, but it must be
    /// greater than zero.
    ///
    /// On success, these system calls return a file descriptor (a nonnegative integer).  On error,
    /// -1 is returned, and errno is set to indicate the error. The error can be found via using
    /// io::Error::last_os_error()
    pub fn epoll_create(size: i32) -> i32;

    /// close a file descriptor we get when we create an epoll instance.
    ///
    /// This is simply to release resources correctly.
    ///
    /// https://man7.org/linux/man-pages/man2/close.2.html
    ///
    /// #include <unistd.h>
    ///
    /// int close(int fd);
    pub fn close(fd: i32) -> i32;

    /// control interface for an epoll file descriptor
    ///
    /// This is the call we make to register our interest in an event.
    /// It supports three operations:
    /// - add a new file descriptor to the epoll instance = EPOLL_CTL_ADD
    /// - modify an existing file descriptor in the epoll instance = EPOLL_CTL_MOD
    /// - remove a file descriptor from the epoll instance = EPOLL_CTL_DEL
    ///
    /// documentation: https://man7.org/linux/man-pages/man2/epoll_ctl.2.html
    ///
    /// #include <sys/epoll.h>
    ///
    /// int epoll_ctl(int epfd, int op, int fd, struct epoll_event *_Nullable event)
    ///
    /// epdf: file descriptor returned by epoll_create
    /// op: one of EPOLL_CTL_ADD, EPOLL_CTL_MOD, EPOLL_CTL_DEL
    /// fd: target file descriptor (Source)
    pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;

    /// wait for an I/O event on an epoll file descriptor (blocking)
    ///
    /// we pass in a &mut Event, that is populated by the notification to inform
    /// use as to what events occured when the thread is woken  up or when it times out.
    ///
    /// https://man7.org/linux/man-pages/man2/epoll_wait.2.html
    pub fn epoll_wait(epfd: i32, events: *mut Event, max_events: i32, timeout: i32) -> i32;
}

pub(crate) fn check(bitmask: i32) {
    const EPOLLIN: i32 = 0x1;
    const EPOLLET: i32 = 1 << 31;
    const EPOLLONESHOT: i32 = 0x40000000;

    let a = bitmask & EPOLLIN;
    let read = bitmask & EPOLLIN != 0;
    let et = bitmask & EPOLLET != 0;
    let oneshot = bitmask & EPOLLONESHOT != 0;

    println!("read event? {read}, edge-triggered? {et}, oneshot? {oneshot}");
    // println!("{EPOLLONESHOT:032b}");
    // println!("{bitmask:032b}");
    // println!("{EPOLLIN:032b}");
    // println!("{a:032b}");
}

#[cfg(not(target_arch = "x86_64"))]
pub(crate) fn print_event_debug(event: &Event) {
    println!("Registering interest in event: {event:?}");
    println!("event.events  (interest) : {:032b}", event.events);
    println!("event.epoll_data (token) : {}", event.epoll_data);
}

#[cfg(target_arch = "x86_64")]
pub(crate) fn print_event_debug(event: &Event) {
    // Below is due to using repr(packed) and unaligned access
    println!("No event debug on x86_64");
}
