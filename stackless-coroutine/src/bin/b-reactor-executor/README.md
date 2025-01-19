# Description

Progresses the work done in `a-runtime`. The aim is to reduce the tight
coupling between the executor and reactor that occurs due to both of these
being aware of `mio::Poll`. This limits the executor to being dependent on a
single reactor implementation and hence being woken for only specific types of
events. 

This is achieved via adding usage of the `Waker` type defined in the Rust
standard library.

Dependencies before:
```
    -------> Future <------
    |                     |
    |                     |
Executor <-------------> Reactor
```

Dependencies after:
```
    -------> Future <------
    |          ^          |
    |          |          |
Executor --> Waker <-- Reactor
```

Now the executor and reactor are not tightly coupled. This enables us to even
use multiple reactors within a single runtime.

⚠️ `corofy` does not know about Wakers, and we need to manually edit the generated
`main_corofy.rs` file. So try not to make further changes to `main_async.rs`.


### Goals
- Add a way for executor to *sleep* and *wake up*, that is not coupled to
  `mio::Poll`.
- Add `Waker` that enables signalling to executor that a task is ready to
  progress.
- Change custom `crate::future::Future` definition to accept a Waker when
  future is polled.

### Usage
Run with following:
```bash
cargo run -p stackless-coroutine --bin b-reactor-executor
```

# Requirements
- `corofy` found within [rust-async-utils][1] (private repo)
- `delayserver` found within [rust-async-utils][1] (private repo)

### corofy
This `build.rs` located at root of workspace member should auto-gen `main_corofy.rs`
whenever changes are detected in `main_async.rs`. For clarity, when you make changes
and write them out in editor, rust-analyzer will run `cargo check` or `cargo clippy`.
This will also cause the build script to run, but cargo will first check to see if
there has been a change in `main_async.rs` and execute the `main` function in the `build.rs`
file.

NOTE: I also disabled following in `rust-analyzer` settings:

in settings.json
```json
    "rust-analyzer.diagnostics.disabled": [
        "unlinked-file"
    ]
```

In nvim config:
```lua

["rust-analyzer"] = {
    diagnostics = {
        disabled = ["unlinked-file"]
    }
}
```


[1]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"


