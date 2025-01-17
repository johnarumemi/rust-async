use std::collections::{HashMap, VecDeque};
use std::thread;
use std::time::Duration;

use stackless_coroutine::prelude::*;

fn hello() {
    println!("Hello world!");
}

fn main() {
    let mut future = coroutine_main();

    loop {
        match future.poll() {
            PollState::NotReady => {
                println!("Waiting for response from delay server...");
                // println!("Schedule other tasks");
            }
            PollState::Ready(_) => {
                break;
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

/// non-leaf pauseable Task
struct Coroutine {
    state: State,
}

/// State of the coroutine / Task
enum State {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

impl Coroutine {
    fn new() -> Self {
        Coroutine {
            state: State::Start,
        }
    }
}

fn coroutine_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

// async fn async_main() {
//     println!("Program starting");

//     let txt = Http::get("/1000/HelloWorld").await;
//     println!("{txt}");

//     let txt2 = Http::get("/500/HelloWorld2").await;
//     println!("{txt2}");
// }

/// Note that our Task / Coroutine is implementing the Future trait here.
impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match &mut self.state {
                State::Start => {
                    println!("Program starting");

                    // Prepare first future (we are not polling it yet) polling
                    let fut = Box::new(Http::get("/1000/HelloWorld"));

                    // Move to next state
                    // note that we haven't polled the future yet.
                    self.state = State::Wait1(fut);
                    // continue loop and enter relevant branch for handling the next state
                }
                State::Wait1(future) => match future.poll() {
                    // we immediately poll the future
                    PollState::NotReady => break PollState::NotReady, // return
                    PollState::Ready(response) => {
                        println!("{response}");

                        // now prepare next future for polling
                        let fut = Box::new(Http::get("/500/HelloWorld2"));
                        self.state = State::Wait2(fut);
                        // continue loop and enter relevant branch for handling the next state
                    }
                },
                State::Wait2(future) => match future.poll() {
                    // we immediately poll the future
                    PollState::NotReady => break PollState::NotReady, // return
                    PollState::Ready(response) => {
                        println!("{response}");

                        // We are done with this Task now, so transition state to Resolved
                        self.state = State::Resolved;

                        // Then return Ready to whoever is polling this future.
                        break PollState::Ready(()); // return
                    }
                },
                State::Resolved => {
                    // poll was called after the future was resolved
                    panic!("Polled a resolved future")
                }
            }
        }
    }
}
