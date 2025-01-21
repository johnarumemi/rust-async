//! Manages the execution of futures.
//!
//! The logic that was initially in `main.rs` in the `a-coroutine` example
//! is essentially shifted to be part of the Runtime's responsibilities.

use std::sync::OnceLock;

use mio::{Events, Poll, Registry};

use crate::future::{Future, PollState};

mod executor;
mod reactor;

pub use executor::{spawn, Executor, Waker};
pub use reactor::reactor;

pub fn init() -> Executor {
    // Start reactor and event_loop
    // NOTE: event looop is spawned in different thread,
    // and reactor is initialised as a global static variable.
    reactor::start();
    // create executor and return it to caller
    Executor::new()
}
