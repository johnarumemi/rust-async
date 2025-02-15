//! Run with following
//! ```bash
//! cargo run -p stackless-coroutine --bin c-coroutines-problem
//! ```
#![allow(unused)]

use std::{
    future::Future,
    io::{ErrorKind, Read, Write},
    pin::Pin,
    task::{Context, Poll},
};

mod future;
mod http;
mod runtime;

use crate::http::Http;
use crate::runtime::{reactor, Executor};

pub fn main() {
    // initialise the runtime
    let mut executor = runtime::init();

    // The main top-level future we start executor with
    let future = async_main();

    executor.block_on(future);
}

async fn async_main() {
    let mut buffer = String::from("\nBUFFER:\n----\n");
    let writer = &mut buffer;

    println!("Program starting");

    let txt = Http::get("/600/HelloAsyncAwait").await;
    println!("{txt}");

    let txt = Http::get("/400/HelloAsyncAwait").await;
    println!("{txt}");
}
