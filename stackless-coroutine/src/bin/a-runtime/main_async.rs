//! Run with following
//! ```bash
//! cargo run -p stackless-coroutine --bin a-runtime
//! ```
#![allow(unused)]


use crate::future::{Future, PollState};
use crate::http::{self, Http};
use crate::runtime::Runtime;

pub fn run() {
    let future = async_main();

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
