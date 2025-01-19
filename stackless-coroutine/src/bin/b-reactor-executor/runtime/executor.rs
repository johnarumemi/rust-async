use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, Thread},
};

use crate::future::{Future, PollState};

/// Alternative is to place this in `future` crate, since it's part of the `Future` trait.
#[derive(Clone)]
pub struct Waker {
    /// Handle to executor thread
    ///
    /// This enables us to park and unpark the executor's thread using the Waker.
    /// WARNING: any other library may also be making use of getting the current thread, parking it
    /// and unparking it. This may cause us to miss wake ups or get trapped in deadlocks. This is
    /// only used for this simple implementation: see other asynchronous libraries for how they
    /// implement their Wakers.
    /// e.g. crossbeam: https://docs.rs/crossbeam/latest/crossbeam/sync/struct.Parker.html
    thread: Thread,
    /// Identifies which Task this waker is associated with. Returned from event_queue ready list as
    /// part user data.
    id: usize,
    /// Reference to the ready_queue of the executor
    ///
    /// usize: represents the id of a Task in the ready queue.
    ///
    /// NOTE: Waker could also have been supplied a function via executor that would
    /// add associated Task back to it's ready queue, without the Waker itself keeping
    /// a reference to the queue directly like below.
    /// TODO: implement above method instead.
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        // 1. Add wakers associated task to ready queue (let executor know it's ready to be polled)
        self.ready_queue
            .lock()
            .as_deref_mut()
            .map(|queue| {
                queue.push(self.id);
            })
            .unwrap();

        // 2.  Unpark executor if it's yielded control back to the OS scheduler / is parked.
    }
}
