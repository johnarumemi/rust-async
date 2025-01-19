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
    reactor::start();
    Executor::new()
}
