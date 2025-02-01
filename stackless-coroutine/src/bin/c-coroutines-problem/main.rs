//! Run with following
//! ```bash
//! cargo run -p stackless-coroutine --bin c-coroutines-problem
//! ```
#![allow(unused)]

use std::fmt::Write;
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
//     let mut buffer = String::from("\nBUFFER:\n----\n");
//     let writer = &mut buffer;
//
//     println!("Program starting");
//
//     let txt = Http::get("/600/HelloAsyncAwait").wait;
//     writeln!(writer, "{txt}").unwrap();
//
//     let txt = Http::get("/400/HelloAsyncAwait").wait;
//     writeln!(writer, "{txt}").unwrap();
//
//     println!("{}", buffer);
//

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output = String> {
    Coroutine0::new()
}

#[derive(Default)]
struct Stack0 {
    buffer: Option<String>,
    /// We can't use a &mut String, since we know that lifetime is tied
    /// to the buffer itself, so Rust wouldn't allow that to work.
    /// Rust does allow a field to whole a reference of &self: self-referential.
    /// Rust has no way to determine the lifetime of such self references.
    ///
    writer: Option<*mut String>,
}

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
                    self.stack.buffer = Some(String::from("\nBUFFER:\n----\n"));

                    self.stack.writer = self.stack.buffer.as_mut().map(|v| v as *mut _);

                    println!("Program starting");

                    // ---------------------------------
                    let fut1 = Box::new(Http::get("/600/HelloAsyncAwait"));
                    self.state = State0::Wait1(fut1);
                }

                State0::Wait1(ref mut f1) => {
                    match f1.poll(waker) {
                        PollState::Ready(txt) => {
                            let writer = unsafe { &mut *self.stack.writer.take().unwrap() };

                            // ---- Code you actually wrote ----
                            writeln!(writer, "{txt}").unwrap();

                            // ---------------------------------
                            let fut2 = Box::new(Http::get("/400/HelloAsyncAwait"));
                            self.state = State0::Wait2(fut2);

                            self.stack.writer = Some(writer);
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Wait2(ref mut f2) => {
                    match f2.poll(waker) {
                        PollState::Ready(txt) => {
                            let buffer = self.stack.buffer.as_ref().unwrap();
                            let writer = unsafe { &mut *self.stack.writer.take().unwrap() };

                            // ---- Code you actually wrote ----
                            writeln!(writer, "{txt}").unwrap();

                            println!("{}", buffer);

                            // ---------------------------------
                            self.state = State0::Resolved;

                            let _ = self.stack.buffer.take();

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
