use crate::future::{Future, PollState};
use mio::{Events, Poll, Registry};
use std::sync::OnceLock;

// Used for registering interest in events.
// We use a static to avoid having to pass in
// references to the registry to all our futures.
static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get().expect("Called outside a runtime context")
}

pub struct Runtime {
    poll: Poll,
}

impl Runtime {
    pub fn new() -> Self {
        let poll = Poll::new().unwrap();

        // get a clone of the poll registry
        let registry = poll.registry().try_clone().unwrap();

        REGISTRY.set(registry).unwrap();

        Self { poll }
    }

    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String>,
    {
        let mut future = future;

        loop {
            // reactor-executor event loop (tighly coupled)
            // executor: Schedules the future and polls
            match future.poll() {
                PollState::NotReady => {
                    // future is not ready, let's block on events
                    println!("Scheduler: Future is not ready, blocking on events...");
                    // below is passed to syscall and is populated with event notifications
                    let mut events: Events = Events::with_capacity(100);

                    // reactor: reacts to events and notifications
                    // block / yield to OS when there is no more work to do atm
                    self.poll.poll(&mut events, None).unwrap();

                    // we reach here when we are woken up and loop back to polling the top level
                    // future.
                }
                PollState::Ready(_) => {
                    // future is ready / completed
                    break;
                }
            }
        }
    }
}
