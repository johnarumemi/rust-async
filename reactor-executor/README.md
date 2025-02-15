# Description


### Goal

Replace our custom Future with the Rust standard library Future and Waker. Also
replace use of our custom preprocessor with standard async/await.

Differences:
- use standard library `Future`
- using `Context` when polling, rather than `Waker`
- using `async/await` rather than corofy
- futures return `Poll` rather than our custom `PollState`

### Usage

Run with following:

```bash
cargo run -p reactor-executor
```

# Requirements
- `delayserver` found within [rust-async-utils][1] (private repo)

[1]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"

