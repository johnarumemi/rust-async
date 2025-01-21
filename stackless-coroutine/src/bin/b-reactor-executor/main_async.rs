//! WARNING: Make code changes in `main_async.rs`. `main_corofy.rs` is
//! genereted from the build script, which reads in `main_async.rs` and
//! passes it to the `corofy` binary.
#![allow(unused)]


use crate::future::{Future, PollState};
use crate::http::{self, Http};
use crate::runtime::{self, Waker};

pub fn run() {
    // initiaise the runtime
    let mut executor = runtime::init();


    // The main top-level future we start executor with
    let future = async_main();

    executor.block_on(future);
}


coroutine fn async_main(){
    println!("Program starting");
    let txt = Http::get("/600/HelloAsyncAwait").wait;
    println!("{txt}");
    let txt = Http::get("/400/HelloAsyncAwait").wait;
    println!("{txt}");

}
