//! WARNING: Make code changes in `main_async.rs`. `main_corofy.rs` is
//! genereted from the build script, which reads in `main_async.rs` and
//! passes it to the `corofy` binary.
#![allow(unused)]


use crate::future::{Future, PollState};
use crate::http::{self, Http};
use crate::runtime::Runtime;

pub fn run() {
    let future = async_main();

    // unlike the a-coroutine example, rather than directly polling the future in a loop,
    // we create a runtime and pass the future to the Runtime. 
    let mut runtime = Runtime::new();
    runtime.block_on(future);
}


coroutine fn async_main(){
    println!("Program starting");
    let txt = Http::get("/600/HelloAsyncAwait").wait;
    println!("{txt}");
    let txt = Http::get("/400/HelloAsyncAwait").wait;
    println!("{txt}");

}
