//! Run with following
//! ```bash
//! cargo run -p stackless-coroutine --bin a-coroutine
//! ```
#![allow(unused)]

use std::time::Instant;

/// future related code
mod future;

/// code for http client
mod http;

use future::{Future, PollState};
use http::{Http, Response};

/// Represents a pause-able / resumable task
///
/// Modelled as a state machine that can be paused and resumed.
/// The implementation we will using for this is much closer to what
/// the rust compiler does with the `async/await` syntax.
struct Coroutine {
    state: CoroutineState,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            state: CoroutineState::Start,
        }
    }
}

/// State of the task that can be polled. It is a state machine.
///
/// # Equivalent syntax is `async/await` were used
/// ```rust
///
/// async fn async_main() {
///     // 1) --> Start state, nothing useful occurred yet
///     println!("Program Starting");
///
///     // 2) >>> Wait1 state, 1st future is being polled
///     let txt = Http::get("/1000/HelloWorld").await;
///     // <<<
///
///
///     // 3) >>> Wait2 state, 2nd future is being polled
///     println!("{txt}");
///     let txt2 = Http::get("500/HelloWorld").await;
///     // <<<
///
///     println!("{txt2}");
///     // 4) >>> Resolved state, both futures are resolved
/// }
///
/// ```
enum CoroutineState {
    /// 1. Initial state
    ///
    /// Corutine is instantiated in this state.
    /// It is not doing any useful computation and is a lazy future.
    /// It is waiting for 1st poll to be done on it.
    ///
    /// Transitions to `Wait1` after the 1st poll
    Start,

    /// 2. Waiting for the 1st future to complete
    ///
    /// state at start: Start
    /// When we poll the the Coroutine, we make the first http get request, which
    /// returns a future. After polling this for the first time, we store the future
    /// in `Wait1` and return control back to the caller.
    ///
    /// Once the Coroutine is polled, it will keep trying to poll Wait1 and return
    /// either Pending (queue coroutine for polling again) or resolve with a `Response`.
    ///
    /// state at end (Pending): `Wait1`
    /// state at end (ready): progresses to `println` statement and then 2nd get request
    Wait1(Box<dyn Future<Output = Response>>),

    /// 3. Waiting for the 2nd future to complete
    ///
    /// state at start: Wait1
    ///
    /// The 2nd http get request we make also returns a future, which we poll
    /// once and return control back to the caller. We store this future in `Wait2`.
    ///
    /// state at end (Pending): `Wait2`
    /// state at end (ready): progresses to execute next `println` statement
    /// and transitions to Resolved.
    ///
    /// Transitions to Resolved after the 3rd poll
    Wait2(Box<dyn Future<Output = Response>>),

    /// 4. Future is Resolved and no further useful work can be done.
    ///
    /// If we try to poll a future that has transitioned to Resolved,
    /// we can get either behaviour:
    /// a) panic
    /// b) return immediately with the resolved value again.
    ///
    /// NOTE: Option (b) can be impossible if the future returned an owned
    /// value when it initially resolved. Hence it no longer makes sense
    /// to return a value from it again.
    Resolved,
}

/// Async task that can be paused and resumed and returns a null value.
///
/// Note that the associated type `Output` is the null value.
/// Since this coroutine modesl the an `async fn async_main() {...}` function, with
/// no return value, the future should also resolve to a null value.
impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        use CoroutineState::*;
        // NOTE: originally this would not be in a loop directly within the poll method (TBC).
        // We use the loop to drive the state machinr forward, until we reach a point where
        // one of the child futures can no longer progress and return a PollState::NotReady.
        // This does have the benefit of being able to yield control back to the caller less often,
        // and only when we absolutely need to.
        // At the same time, if we didn't do this, we could never get to a point whereby
        // we reach the OSes event_queue and register with it to notify the Reactor when
        // a future is ready to be polled again.
        // If Reactor is not notified, it will not poll the future again.
        // This is not an issue for this initial implementation, since we don't use a full
        // reactor-executor model

        loop {
            match &mut self.state {
                // 1. Initial state
                Start => {
                    println!("Program Starting");

                    let future = Http::get("/1000/HelloWorld");

                    // 2. Transition to Wait1 state
                    // store future, change state and poll future
                    // store state and poll
                    self.state = Wait1(Box::new(future));

                    // continue in loop
                }
                Wait1(ref mut future) => match future.poll() {
                    PollState::Ready(Response { body: txt }) => {
                        // Future has resolved with a response, we move to print instruction
                        println!("{txt}");

                        // then we move on to next get request
                        let future = Http::get("/500/HelloWorld");

                        // 3. Transition to Wait2 state
                        self.state = Wait2(Box::new(future))

                        // continue in loop
                    }
                    // If we are not ready, we yield control back to caller.
                    // They can choose to poll us again at some future time.
                    PollState::NotReady => break PollState::NotReady,
                },
                Wait2(ref mut future) => match future.poll() {
                    PollState::Ready(Response { body: txt }) => {
                        // Future has resolved with a response, we move to print instruction
                        println!("{txt}");

                        // There is nothing else to do in the async function.
                        // We can transition the future to a Resolved state.
                        // There is nothing further to return either as the Coroutine
                        // resolves to a null value.
                        self.state = Resolved;
                        break PollState::Ready(());
                    }
                    // If we are not ready, we yield control back to caller.
                    // They can choose to poll us again at some future time.
                    PollState::NotReady => break PollState::NotReady,
                },
                Resolved => {
                    // Future is resolved, we will panic if polled again
                    panic!("Future is already resolved!");
                }
            }
        }
    }
}

fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

/// Main function that drives the state machine.
///
/// Uses a simple loop to drive the state machine forward.
fn main() {
    let mut future = async_main();

    while let PollState::NotReady = future.poll() {
        println!("Current future is not ready. Schedule other tasks.");

        // For now we will just sleep for 100ms and then poll the future again.
        // This simulates the OSes event loop, where we would wait for some event
        // to occur / be ready before polling the future again.
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
