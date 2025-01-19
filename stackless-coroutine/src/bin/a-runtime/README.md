# Description
Progresses the work done in `a-coroutine`. The aim is to improve upon `a-coroutine` by
avoiding having to use a sleep to contiously poll the future to determine if it's ready
to make progress. Instead we will rely on a reactor and it's associated OS event_queue
to notify an executor when a task is ready to be scheduled to be polled again. This
should reduce the amount of busy looping we do to keep polling the future.

### Usage
Run with following:
```bash
cargo run -p stackless-coroutine --bin a-runtime
```

### Compromises
These are to limit the scope of the example produced here.
- Avoid error handling
- Use concrete types and not generics
- Avoid macros

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

