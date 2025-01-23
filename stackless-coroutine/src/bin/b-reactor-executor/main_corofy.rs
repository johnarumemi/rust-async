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

    let mut handles = vec![];

    for i in 1..12 {
        let name = format!("executor-{}", i);
        let h = Builder::new().name(name).spawn(move || {
            let mut executor = Executor::new();

            // The main top-level future we start executor with
            let future = async_main();
            executor.block_on(future);
        }).unwrap();

        handles.push(h)
    }

    // The main top-level future we start executor with
    let future = async_main();

    executor.block_on(future);

    handles.into_iter().for_each(|h| h.join().unwrap());
}







// =================================
// We rewrite this:
// =================================
    
// coroutine fn request(i: usize) {
//     let path = format!("/{0}/HelloWorld{0}", i * 1000);
//     let txt = Http::get(&path).wait;
//     println!("{txt}");

// }

// =================================
// Into this:
// =================================

fn request(i: usize) -> impl Future<Output=String> {
    Coroutine0::new(i)
}
        
enum State0 {
    Start(usize),
    Wait1(Box<dyn Future<Output = String>>),
    Resolved,
}

struct Coroutine0 {
    state: State0,
}

impl Coroutine0 {
    fn new(i: usize) -> Self {
        Self { state: State0::Start(i) }
    }
}


impl Future for Coroutine0 {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
        match self.state {
                State0::Start(i) => {
                    // ---- Code you actually wrote ----
                    let path = format!("/{0}/HelloWorld{0}", i * 1000);

                    // ---------------------------------
                    let fut1 = Box::new( Http::get(&path));
                    self.state = State0::Wait1(fut1);
                }

                State0::Wait1(ref mut f1) => {
                    match f1.poll(waker) {
                        PollState::Ready(txt) => {
                            // ---- Code you actually wrote ----
                            println!("{txt}");

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


// =================================
// We rewrite this:
// =================================
    
// coroutine fn async_main(){
//     println!("Program starting");
// 
//     for i in 0..=5 {
//         let future = request(i);
// 
//         runtime::spawn(future);
//     }

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output=String> {
    Coroutine1::new()
}
        
enum State1 {
    Start,
    Resolved,
}

struct Coroutine1 {
    state: State1,
}

impl Coroutine1 {
    fn new() -> Self {
        Self { state: State1::Start }
    }
}


impl Future for Coroutine1 {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
        match self.state {
                State1::Start => {
                    // ---- Code you actually wrote ----
                    println!("Program starting");

    for i in 0..=5 {
        let future = request(i);

        runtime::spawn(future);
    }

                    // ---------------------------------
                    self.state = State1::Resolved;
                    break PollState::Ready(String::new());
                }

                State1::Resolved => panic!("Polled a resolved future")
            }
        }
    }
}
