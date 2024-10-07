//! M:N threading model
//!
//! Where there is a runtime responsible for deciding what
//! thread/task(M) gets to run on OS thread (N).
//!
//! Here "Threads" are an abstraction that represent a
//! thread of execution / task within our runtime. They are
//! not OS threads.
//!
//! These are User threads created and managed by our
//! runtime / scheduler and OS has no knowledge of them. To
//! the OS, it is merely scheduling the OS threads we spawn
//! within the process, and scheduling them on the CPU.
//!
//! Since we yield to our scheduler and not the OS
//! scheduler, we can schedule another task/thread to
//! progress immediately.
//!
//! From the OS's perspective, our OS threads are
//! continously busy and it will avoid pre-empting them as
//! much as possible.
#![feature(naked_functions)]
use std::arch::asm;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2; // 2 MB
const MAX_THREADS: usize = 4;

static mut RUNTIME: usize = 0; // pointer to our runtime

/// Main entrypoint for our runtime
pub struct Runtime {
    threads: Vec<Thread>,

    /// Thread we are currently running
    current: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    /// Available and ready to be assigned a task
    Available,
    /// Thread is running
    Running,
    /// Thread is ready to move forward and resume execution
    Ready,
}

/// Holds data for a thread
struct Thread {
    // holds the tasks / threads current execution state (not CPU state)
    stack: Vec<u8>,
    // stores the CPU state of the thread
    ctx: ThreadContext,
    state: State,
    base: usize,
}

fn offset(rsp: u64, base: usize) -> usize {
    // below assumes base is at a higher memory address than rsp
    base.saturating_sub(rsp as usize)
}

/// Represents our CPU state
///
/// Context the CPU needs to resume where it left off
/// on a stack and a state field that holds our thread state.
///
/// # Context Stored
///
/// Holds `caller saved registers` and the callee
/// needs to restore them before the caller is resumed.
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    // stack pointer
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

impl Thread {
    // initialize data to a newly created thread
    fn new() -> Self {
        // thread has it's own stack
        Thread {
            // pre-allocate 2MB of memory for our stack.
            // This is sub-optimal as we should allocate only on first use of thread.
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
            base: 0,
        }
    }
}

impl Runtime {
    /// Create a new runtime
    ///
    /// This runtime is created with MAX_THREADS available, one of which is the main / base thread.
    /// The base thread is always running and is the first thread to be created.
    ///
    /// This means that we do not create threads only when and as needed.
    pub fn new() -> Self {
        let base_thread = Thread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running, // Set thread as running
            base: 0,
        };

        let mut threads = vec![base_thread];

        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();

        threads.append(&mut available_threads);

        Self {
            threads,
            current: 0,
        }
    }

    /// Initialize static RUNTIME
    ///
    /// This allows the RUNTIME to be accessed from anywhere in our code.
    ///
    /// This is not safe and we must make sure that the address to &self never changes during
    /// program execution, or else RUNTIME will contain and invalid address. i.e. we can't do
    /// anything that will invalidate the pointer.
    pub fn init(&self) {
        unsafe {
            let ptr: *const Runtime = self;
            RUNTIME = ptr as usize;
        }
    }

    /// Being main program loop
    pub fn run(&mut self) -> ! {
        println!("Main Loop Starting");
        while self.t_yield() {
            println!("Main Loop Calling Yield on base thread again...")
        }
        std::process::exit(0);
    }

    /// return function called when thread is finished
    ///
    /// user of thread does not call this. We setup stack so that it is called
    /// when the task is done.
    ///
    /// This is not called for base_thread, our runtime does that for us
    /// via calling t_yield on the base_thread.
    ///
    /// This function is called from the `guard` function, which is
    /// manually written to the threads stack as part of the custom epilogue.
    fn t_return(&mut self) {
        if self.current != 0 {
            println!("Returning thread {} and setting to Available", self.current);
            // let runtime know thread is ready to be assigned a new task
            // as it is completed Running of previous task assigned to it.
            self.threads[self.current].state = State::Available;

            // schedule a new thread to be run
            self.t_yield();
        }
        // don't do anything for base thread, our runtime handles
        // calling that on base thread for us at start in the `run` method
    }

    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        println!("Yielding thread {}", self.current);
        // # 1. Scheduler
        let mut pos = self.current;

        // find a thread that is in Ready state
        // and can be progressed
        while self.threads[pos].state != State::Ready {
            pos += 1;

            if pos == self.threads.len() {
                pos = 0; // base thread
            }

            if pos == self.current {
                return false;
            }
        }
        // we have found a Ready thread, indexed by `pos`

        // If current thread is in Available state, it has no task to even run
        // so nothing is done to it.
        if self.threads[self.current].state != State::Available {
            // If current thread is `Running` (from `yield_thread` usage), then
            // we can simply transition from `Running` to `Ready`. This effecitevly
            // adds it back to list of threads to be scheduled for running, since they
            // have an active task still to complete and have not returned.
            //
            // There probably isn't any instance when this branch is selected
            // where current thread is Ready, that doesn't make sense from
            // how all this code has been setup.
            self.threads[self.current].state = State::Ready
        }

        // Set new thread's state as Running (we are about to switch context into it)
        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        // # 2. Context Switch
        unsafe {
            let old_ctx: *mut ThreadContext = &mut self.threads[old_pos].ctx;
            let new_ctx: *const ThreadContext = &self.threads[pos].ctx;

            // call swith to save current context (old_ctx)
            // and load new context into CPU (new_ctx).
            //
            // New context is either a new task or all the context the CPU needs
            // to resume work on an existing task.
            if old_pos != pos {
                println!("switching from thread {} to {}", old_pos, pos);
            }
            // `clobber_abi("C")` tells the compiler that the `switch` function
            // will modify the registers in a way that it can't predict.
            // So the compiler should not make any assumptions about the values
            // of registers after the call to `switch` function.
            //
            // This means the compiler will emit instructions to push registers
            // it uses to the stack before the call to `switch` and pop them
            // them resuming after the `asm!` block.
            //
            // Not that the switch function, switches context, but we only store
            // the callee saved registers. There may be many other general purpose
            // registers currently being used.
            //
            // We have marked the switch function as [naked], so it doesn't
            // automatically save and restore caller registers. So to be safe,
            // we explicitly let the compiler know about this in the `asm!` block.

            // When we save our context below, we are essentially saving the CPU state
            // at the moment in time we are in this function itself.
            // When we resume, (another thread yields back to us), it will be
            // after our asm! block.
            asm!(
                "call switch",
                in("rdi") old_ctx,
                in("rsi") new_ctx,
                clobber_abi("C")
            );
        }

        // # 3. Resume Execution here after another thread context switches to us.
        //
        // After a thread has yielded to another thread, when it is next resumed,
        // it should resume from here.
        // i.e. if in thread 1 we call
        // ```rust
        // Before
        // yield_thread();
        // After
        // ```
        // Then on restoring execution of this threads context, we resume from here
        // and eventually reach the "After" section.
        //
        // Note that for the base thread, it will return from this function and the
        // main event loop will call yield on it again.
        println!(
            "Returning from yield in thread {}, had previously switched to: {}",
            self.current, pos
        );

        // Below stops compiler from optimising our code away somehow.
        self.threads.len() > 0
    }

    /// Spawn a new task onto an available thread
    ///
    /// panics if no available thread found
    pub fn spawn(&mut self, f: fn()) {
        // find available thread
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("no available thread");

        let size = available.stack.len();

        // initialise thread's stack
        unsafe {
            // set stack pointer to point to bottom of our stack / u8 byte array
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);

            // ensure memory segment is 16byte aligned
            let s_ptr_before = s_ptr as usize;
            let s_ptr = (s_ptr as usize & !15) as *mut u8;

            if s_ptr_before != (s_ptr as usize) {
                println!(
                    "Stack was re-aligned, before: {}, after: {}",
                    s_ptr_before, s_ptr as usize
                );
            }

            available.base = s_ptr as usize;
            // write out function pointers / address to our stack in order
            // call order:
            // 1. `f` -> function to run concurrently
            // 2. `skip` -> skip to next instruction (it's just a `ret instruction`)
            // 3. `guard` -> set current thread state to Available and schedul next thread
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);

            // store stack pointer for thread such that it's pointing at `f`
            available.ctx.rsp = s_ptr.offset(-32) as u64;
        }

        // Set thread as ready
        available.state = State::Ready;
    }
}

fn guard() {
    unsafe {
        // get mutable raw pointer
        let rt_ptr = RUNTIME as *mut Runtime;
        // dereference raw pointer to get Runtime and call t_return
        //
        // This will set the current thread state to Available
        // and call t_yield() to schedule next thread to run
        (*rt_ptr).t_return();
    }
}

/// Single instruction to skip to next instruction
#[naked]
unsafe extern "C" fn skip() {
    asm!("ret", options(noreturn));
}

/// A thread can decide that it can no longer make progress an yield execution to another thread.
/// It will still be in the RUNNING state, and it's function has not yet `returned`, so `guard`
/// function has not yet been called (i.e. t_return not called yet).
pub fn yield_thread() {
    unsafe {
        // get mutable raw pointer
        let rt_ptr = RUNTIME as *mut Runtime;

        // dereference raw pointer to get Runtime and call t_yield
        //
        // Let's use call t_yield on our Runtime from an arbitrary place in our code
        // without needing any references to it.
        (*rt_ptr).t_yield();
    }
}

// rdi = pointer into 'old' thread context
// rsi = pointer into 'new' thread context
//
// 1. Read out the values of all the registers we need
// 2. Set all the register values to the register values we saved when we suspended execution
//    on the new thread.

// struct ThreadContext {
//     rsp: u64,  [rdi + 0x00]
//     r15: u64,  [rdi + 0x08]
//     r14: u64,  [rdi + 0x10]
//     r13: u64,  [rdi + 0x18]
//     r12: u64,  [rdi + 0x20]
//     rbx: u64,  [rdi + 0x28]
//     rbp: u64,  [rdi + 0x30]
// }
#[naked]
#[no_mangle]
#[cfg_attr(target_os = "macos", export_name = "\x01switch")] // see: How-to-MacOS-M.md for explanation
unsafe extern "C" fn switch() {
    asm!(
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "ret",
        options(noreturn)
    );
}

fn main() {
    let mut runtime = Runtime::new();

    runtime.init();

    // spawn a task onto an available thread
    runtime.spawn(|| {
        // technically speaking, we have no idea what thread this function is
        // executing on, so we can't really say it's thread 1.
        println!("THREAD 1 STARTING");
        let id = 1;
        // we simply print out a message and yield at end of each iteration
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD 1 FINISHED");
    });

    runtime.spawn(|| {
        println!("THREAD 2 STARTING");
        // task / thread id
        let id = 2;

        // we simply print out a message and yield at end of each iteration
        for i in 0..15 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD 2 FINISHED");
    });
    runtime.run();
}
