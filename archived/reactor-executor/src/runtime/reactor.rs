use crate::runtime::Waker;
use mio::{net::TcpStream, Events, Interest, Poll, Registry, Token};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread,
};

/// maps task_id -> Waker
type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;

static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside a runtime context")
}

pub struct Reactor {
    /// HashMap of Waker objects, each identified by an integer
    wakers: Wakers,
    /// allows interaction with event queue in mio
    registry: Registry,

    /// tracks next available id, so that we can track
    /// which event occured and which waker should be
    /// woken.
    next_id: AtomicUsize,
}

impl Reactor {
    /// Register interest
    ///
    /// `id` is used to determine which event we are
    /// interested in. In our case, it is associated
    /// with a Waker's / Task's id.
    pub fn register(&self, stream: &mut TcpStream, interest: Interest, id: usize) {
        self.registry.register(stream, Token(id), interest);
    }

    pub fn deregister(&self, stream: &mut TcpStream, id: usize) {
        self.wakers.lock().map(|mut w| w.remove(&id)).unwrap();

        self.registry.deregister(stream);
    }

    /// Insert or replace Waker for a given id
    ///
    /// We must always use the latest Waker for a given
    /// `id`.
    pub fn set_waker(&self, waker: &Waker, id: usize) {
        let _ = self
            .wakers
            .lock()
            .map(|mut w| w.insert(id, waker.clone()).is_none())
            .unwrap();
    }

    /// Gets the current `next_id` and increments the value atomically.
    ///
    /// Relaxed ordering does not impose any ordering
    /// constraints, only that the single operation
    /// being executed is atomic.
    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

fn event_loop(mut poll: Poll, wakers: Wakers) {
    let mut events = Events::with_capacity(100);
    loop {
        // block on OS event queue that Poll is
        // associated with.
        poll.poll(&mut events, None);

        // on wakup, dispatch events
        for e in events.iter() {
            let Token(id) = e.token();
            let wakers = wakers.lock().unwrap();

            // Guard against waker having been removed,
            // via a call to the `deregister` Reactor
            // method.
            if let Some(waker) = wakers.get(&id) {
                waker.wake()
            }

            // drop MutexGuard
        }
    }
}

/// Initializes and starts the runtime
pub fn start() {
    use thread::spawn;

    let wakers = Arc::new(Mutex::new(HashMap::new()));

    // OS event queue
    let poll = Poll::new().unwrap();

    // get handle to registry for OS event queue
    let registry = poll.registry().try_clone().unwrap();

    // initial task / waker id
    let next_id = AtomicUsize::new(1);

    // Create a reactor
    let reactor = Reactor {
        wakers: wakers.clone(),
        registry,
        next_id,
    };

    // Reactor is available from all threads, but it's
    // event loop is only running in a specific thread.
    REACTOR.set(reactor).ok().expect("Reactor already running");

    // Run the reactor in it's own dedicated OS thread
    //
    // note that the Poll instance is now owned by the
    // OS thread running the reactor.
    spawn(move || event_loop(poll, wakers));
}
