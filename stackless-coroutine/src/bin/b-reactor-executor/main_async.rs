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


coroutine fn request(i: usize) {
    let path = format!("/{0}/HelloWorld{0}", i * 1000);
    let txt = Http::get(&path).wait;
    println!("{txt}");
}

coroutine fn async_main(){
    println!("Program starting");

    for i in 0..=5 {
        let future = request(i);

        runtime::spawn(future);
    }
}
