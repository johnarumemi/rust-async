//! WARNING: Make code changes in `main_async.rs`. `main_corofy.rs` is
//! genereted from the build script, which reads in `main_async.rs` and
//! passes it to the `corofy` binary.
#![allow(unused)]

use std::thread::Builder;

use crate::future::{Future, PollState};
use crate::http::{self, Http};
use crate::runtime::{self, Executor, Waker};

pub fn run() {
    // initiaise the runtime
    let mut executor = runtime::init();


    // The main top-level future we start executor with
    let future = async_main();

    executor.block_on(future);
}




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

fn async_main() -> impl Future<Output=String> {
    Coroutine0::new()
}
        
enum State0 {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

struct Coroutine0 {
    state: State0,
}

impl Coroutine0 {
    fn new() -> Self {
        Self { state: State0::Start }
    }
}


impl Future for Coroutine0 {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
        match self.state {
                State0::Start => {
                    // ---- Code you actually wrote ----
                    let mut counter = 0;
    println!("Program starting");


                    // ---------------------------------
                    let fut1 = Box::new( Http::get("/600/HelloAsyncAwait"));
                    self.state = State0::Wait1(fut1);
                }

                State0::Wait1(ref mut f1) => {
                    match f1.poll(waker) {
                        PollState::Ready(txt) => {
                            // ---- Code you actually wrote ----
                            println!("{txt}");

    let mut counter += 1;

                            // ---------------------------------
                            let fut2 = Box::new( Http::get("/400/HelloAsyncAwait"));
                            self.state = State0::Wait2(fut2);
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Wait2(ref mut f2) => {
                    match f2.poll(waker) {
                        PollState::Ready(txt) => {
                            // ---- Code you actually wrote ----
                            println!("{txt}");

    let mut counter += 1;
    println!("Received {} responses.", counter);


                            // ---------------------------------
                            self.state = State0::Resolved;
                            break PollState::Ready(String::new());
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Resolved => panic!("Polled a resolved future")
            }
        }
    }
}
