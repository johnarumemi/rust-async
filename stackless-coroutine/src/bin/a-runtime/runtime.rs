//! Manages the execution of futures.
//!
//! The logic that was initially in `main.rs` in the `a-coroutine` example
//! is essentially shifted to be part of the Runtime's responsibilities.
use std::sync::OnceLock;

use mio::{Events, Poll, Registry};

use crate::future::{Future, PollState};

/// Registry is used for registering interest in events on a source.
///
/// When HttpGetFuture makes a non-blocking IO request, it should
/// register interest on read events on the streams file descriptor.
///
/// # OnceLock<T>
/// OnceLock is used to ensure static can only be written to once.
/// Useful for Singletons.
pub static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    // we expect the runtime, on initialisation, to set the REGISTRY static variable.
    REGISTRY
        .get()
        .expect("Registry not initialized. Called outside a runtime context.")
}

pub struct Runtime {
    /// Abstraction over the OS'es event_queue polling / select interface
    poll: Poll,
}

impl Runtime {
    pub fn new() -> Self {
        // create a new poll instance and also the underlying OS event queue.
        let poll = Poll::new().unwrap();

        // get a clone to the poll's registry to set global registry.
        // This is now a registry handle owned by the runtime!
        let registry = poll.registry().try_clone().unwrap();

        // set the global REGISTRY static variable
        REGISTRY
            .set(registry)
            .expect("Failed to set REGISTRY static variable");

        Self { poll }
    }
    /// The `block_on` method is used to run the future to completion.
    ///
    /// It represents the original `main` function in the `a-coroutine` example.
    /// NOTE: this implementation does not support multiple top-level futures.
    pub fn block_on<F>(&mut self, mut future: F)
    where
        // corofy only supports futures resolving with strings
        F: Future<Output = String>,
    {
        // Remember, out top-level future will return Ready only when all child futures have
        // resolved and return PollState::Ready.
        while let PollState::NotReady = future.poll() {
            println!("\nCurrent future is not ready. Schedule other tasks.");

            // rather than sleep, we block on the event_queue (epoll or kqueue syscalls) with no
            // timeout specified. It is the responsibility of HttpGetRequest to ensure
            // it registers interest on a source when it makes a non-blocking IO request.
            let mut events = Events::with_capacity(100);
            self.poll.poll(&mut events, None);
            println!("Woken up from poll. Checking for ready tasks.\n");
        }
    }
}
