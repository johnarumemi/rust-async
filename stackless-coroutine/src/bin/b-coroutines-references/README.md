# Description

Progresses the work done in `a-coroutine-variables`. 

### Goal

The previous example used a Stack with state containing owned values. 
This new example (`b-coroutines-references`) involves storing references
and accessing these across state changes.

### Usage

Run with following:

```bash
cargo run -p stackless-coroutine --bin b-coroutines-references
```

# Requirements
- `delayserver` found within [rust-async-utils][1] (private repo)

[1]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"

