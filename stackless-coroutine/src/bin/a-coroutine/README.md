# Description
An example of a hand-written coroutine. 

The await / async keywords in Rust provide us with ergonomic ways of create state
machines that represent some pause-able and resumable task that yields values to it's
caller. This example will roll this back and manually create these state machines to
provide an understanding of what is going on under the hood.

### Implements the following
- A simplified `Future` trait
- A simple HTTP client that can only make GET requests
- A task we can pause and resume implemented as a state machine
- A simplified `async / await` syntax called `coroutine / wait`
- A homemade preprocessor to transform `coroutine/wait` functions into 
  state machines the same way `async/await` is transformed.

### Compromises
These are to limit the scope of the example produced here.
- Avoid error handling
- Use concrete types and not generics
- Avoid macros

# Requirements
- `corofy` found within [rust-async-utils][1] (private repo)
- `delayserver` found within [rust-async-utils][1] (private repo)

[1]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"


