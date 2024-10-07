use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, Thread},
};

use crate::future::{Future, PollState};

type Task = Box<dyn Future<Output = String>>;

// Lets us define a static variable that's unique to
// the thread it's called from. This means that all
// threads created will have their own instance and
// it's impossible for one thread to access another
// thread's CURRENT_EXEC variable.
thread_local! {
    static CURRENT_EXEC: ExecutorCore = ExecutorCore::default();
}

#[derive(Default)]
struct ExecutorCore {
    /// Below is not thread safe and should not be
    /// passed between threads.
    /// RefCell allows for interior mutability, but
    /// tracks shared and exclusive references to it's
    /// interior value.
    ///
    /// Holds all the top-level tasks associated with
    /// the current executor (remember that executors
    /// are not work stealing in current implementation)
    ///
    /// Since the ExecutorCore is behind a static
    /// variable, we need interior mutability, provided
    /// by the RefCell.
    tasks: RefCell<HashMap<usize, Task>>,
    ready_queue: Arc<Mutex<Vec<usize>>>,
    // Cell allows for interior mutability
    next_id: Cell<usize>,
}

#[derive(Clone)]
pub struct Waker {
    /// Handle to the thread running the executor for
    /// the associated task.
    /// There can multiple threads running, each with
    /// it's own executor. Executors are not work
    /// stealing, so task's are specific to an
    /// executor.
    thread: Thread,

    /// ID of the task this waker is associated with
    id: usize,

    /// Tasks that are ready to be polled. It is a
    /// reference that can be shared between threads /
    /// wakers.
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        // be careful of calling unpark before
        // mutexguard is dropped.
        let mut queue = self
            .ready_queue
            .lock()
            .map(|mut v| v.push(self.id))
            .unwrap();

        self.thread.unpark();
    }
}

/// Spawn a future onto Executor
///
/// This will add the future as a Task to ExecutorCore
/// and also queue it for polling by the executor.
pub fn spawn<F>(future: F)
where
    F: Future<Output = String> + 'static,
{
    CURRENT_EXEC.with(|e| {
        let id = e.next_id.get();
        e.tasks.borrow_mut().insert(id, Box::new(future));

        // keep guard until we have set the next id
        if let Ok(mut guard) = e.ready_queue.lock() {
            guard.push(id);
            e.next_id.set(id + 1);
        }
    })
}

/// Unit struct for the executor
///
/// All the needed state is already in ExecutorCore, so
/// Executor doesn't need any state.
///
/// You can think of this as being a container to
/// various helper methods for the threads
/// ExecutorCore;
pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    /// Pop next task from ready queue to poll
    ///
    /// LIFO queue, since we append to back pop from
    /// back.
    fn pop_ready(&self) -> Option<usize> {
        CURRENT_EXEC.with(|e| e.ready_queue.lock().map(|mut q| q.pop()).unwrap())
    }

    // Remove a given task from the hashmap
    //
    // If a task returns NotReady, we need to remember
    // to add it back to the `tasks` hashmap.
    fn get_future(&self, id: usize) -> Option<Task> {
        CURRENT_EXEC.with(|e| e.tasks.borrow_mut().remove(&id))
    }

    /// Get waker for current executor / thread
    ///
    /// # Arguments
    /// id: ID of task this waker is associated with
    fn get_waker(&self, id: usize) -> Waker {
        Waker {
            id,
            thread: thread::current(),
            ready_queue: CURRENT_EXEC.with(|e| e.ready_queue.clone()),
        }
    }

    /// Insert a new task, with an id, into the executors hashmap.
    ///
    /// Does not schedule it for polling
    fn insert_task(&self, id: usize, task: Task) {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().insert(id, task));
    }

    /// Number of registered tasks for executor
    fn task_count(&self) -> usize {
        CURRENT_EXEC.with(|q| q.tasks.borrow().len())
    }

    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String> + 'static,
    {
        spawn(future);

        loop {
            // pop a task from ready queue to poll
            while let Some(id) = self.pop_ready() {
                // get the task / future associated with the id.
                // note that this also removes it from
                // the executor's task. Need to add it
                // back in if it returns NotReady.
                let mut future = match self.get_future(id) {
                    Some(f) => f,
                    // guard against false wakeups
                    None => continue,
                };

                // Create a new waker that is specific
                // to task (id) and current executor /
                // thread.
                let waker = self.get_waker(id);

                match future.poll(&waker) {
                    PollState::NotReady => self.insert_task(id, future),
                    PollState::Ready(_) => continue,
                }
            }

            // reach here when there are no more
            // futures ready to be polled from queue.

            // number of uncompleted tasks
            let task_count = self.task_count();
            let name = thread::current().name().unwrap_or_default().to_string();

            if task_count > 0 {
                println!("{name}: {task_count} pending tasks. Sleep until notified.");
                thread::park();
            } else {
                // no tasks left to poll on executor
                println!("{name}: All tasks are finished");
                break;
            }
        }
    }
}
