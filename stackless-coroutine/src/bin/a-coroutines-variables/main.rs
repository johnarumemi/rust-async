//! Run with following
//! ```bash
//! cargo run -p stackless-coroutine --bin a-coroutines-variables
//! ```
#![allow(unused)]

use std::thread::Builder;

mod future;
mod http;
mod runtime;

use crate::future::{Future, PollState};
use crate::http::Http;
use crate::runtime::{Executor, Waker};

pub fn main() {
    // initialise the runtime
    let mut executor = runtime::init();

    // The main top-level future we start executor with
    let future = async_main();

    executor.block_on(future);
}

// NOTE: for this particular example, we generate main_corofy.rs
// and copy it's contents to below section. We then make remainder
// of the alterations within this file and do not use `main_async.rs`
// or `main_corofy.rs` further.

// =================================
// We rewrite this:
// =================================

// coroutine fn async_main(){
//     let mut counter = 0;
//     println!("Program starting");
//
//     let txt = Http::get("/600/HelloAsyncAwait").wait;
//     println!("{txt}");
//
//     let mut counter += 1;
//     let txt = Http::get("/400/HelloAsyncAwait").wait;
//     println!("{txt}");
//
//     let mut counter += 1;
//     println!("Received {} responses.", counter);
//

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output = String> {
    Coroutine0::new()
}

// NEW: Keep a stack for the coroutine to enable
// persisting state across yield points.
#[derive(Default)]
struct Stack0 {
    counter: Option<usize>,
}

/// Holds the various states that the coroutine will transition between
enum State0 {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

struct Coroutine0 {
    state: State0,
    stack: Stack0,
}

impl Coroutine0 {
    fn new() -> Self {
        Self {
            state: State0::Start,
            stack: Stack0::default(),
        }
    }
}

impl Future for Coroutine0 {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
            match self.state {
                State0::Start => {
                    // ---- Code you actually wrote ----
                    println!("Program starting");

                    // NEW: initialise stack (hoist variable declarations to top of function)
                    self.stack.counter = Some(0);

                    // ---------------------------------
                    let fut1 = Box::new(Http::get("/600/HelloAsyncAwait"));
                    self.state = State0::Wait1(fut1);
                }

                State0::Wait1(ref mut f1) => {
                    match f1.poll(waker) {
                        PollState::Ready(txt) => {
                            // ---- Code you actually wrote ----

                            // NEW: restore stack
                            let mut counter = self.stack.counter.take().unwrap();
                            println!("{txt}");

                            // NEW: mutate counter
                            counter += 1;

                            // ---------------------------------
                            let fut2 = Box::new(Http::get("/400/HelloAsyncAwait"));
                            self.state = State0::Wait2(fut2);

                            // NEW: save stack
                            self.stack.counter = Some(counter)
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Wait2(ref mut f2) => {
                    match f2.poll(waker) {
                        PollState::Ready(txt) => {
                            // ---- Code you actually wrote ----

                            // NEW: restore stack
                            let mut counter = self.stack.counter.take().unwrap();

                            println!("{txt}");

                            // NEW: mutate counter
                            counter += 1;

                            println!("Received {} responses.", counter);

                            // ---------------------------------
                            self.state = State0::Resolved;

                            // NEW: save stack (all variables set to None)
                            self.stack.counter = None;

                            break PollState::Ready(String::new());
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Resolved => panic!("Polled a resolved future"),
            }
        }
    }
}
