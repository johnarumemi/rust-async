# Description

Progresses the work done in `b-coroutines-references`. 

### Goal

Simple exploration of the problems wrt to self-referential structs and how this
leads to requiring pinning when implementing stackless coroutines in Rust.

### Usage

Run with following:

```bash
cargo run -p stackless-coroutine --bin b-coroutines-problem
```

# Requirements
- `delayserver` found within [rust-async-utils][1] (private repo)

[1]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"

