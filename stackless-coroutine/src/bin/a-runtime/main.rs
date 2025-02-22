//! Run with following
//! ```bash
//! cargo run -p stackless-coroutine --bin a-runtime
//! ```
#![allow(unused)]

mod future;
mod http;
mod main_corofy;
mod runtime;

#[cfg(test)]
mod main_async;

use future::{Future, PollState};
use runtime::Runtime;

fn main() {
    main_corofy::run();
}
