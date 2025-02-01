# Description

Progresses the work done in `b-reactor-executor`. 

### Goal

The aim is to gain an understanding of pinning and self-referential structs.
First step is to understand that the previous implementations did not allow
keeping state across yield points. The CoroutineState enum only stored futures
to poll for each specific state. What we need to do now, is introduce storing
of state within a `stack` for each coroutine we create.

So `Coroutine` now has 2 fields:
- State
- Stack

### Usage

Run with following:

```bash
cargo run -p stackless-coroutine --bin a-coroutines-variables
```

# Requirements
- `delayserver` found within [rust-async-utils][1] (private repo)

[1]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"

