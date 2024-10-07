mod executor;
mod reactor;

pub use executor::{spawn, Executor, Waker};
pub use reactor::reactor;

pub fn init() -> Executor {
    // start Reactor event_loop in a separate OS thread
    reactor::start();

    // return thread local ExecutorCore
    Executor::new()
}
