//! build script for `main.rs`
//!
//! This is to enable corofy to be used for rewriting
//! the `coroutine/wait` syntax into a state machine.
//!
//! # corofy usage
//!
//! ```
//! corofy [src_path] [optional-dest-path]
//! ```
use std::process::Command;

fn main() {
    Command::new("corofy")
        .arg("src/bin/a-runtime/main_async.rs")
        .arg("src/bin/a-runtime/main_corofy.rs")
        .output()
        .expect("Failed to run corofy for `a-runtime");

    Command::new("corofy_waker")
        .arg("src/bin/b-reactor-executor/main_async.rs")
        .arg("src/bin/b-reactor-executor/main_corofy.rs")
        .output()
        .expect("Failed to run corofy for b-reactor-executor");

    Command::new("corofy_waker")
        .arg("src/bin/a-coroutines-variables/main_async.rs")
        .arg("src/bin/a-coroutines-variables/main_corofy.rs")
        .output()
        .expect("Failed to run corofy for a-coroutines-variables");

    Command::new("corofy_waker")
        .arg("src/bin/b-coroutines-references/main_async.rs")
        .arg("src/bin/b-coroutines-references/main_corofy.rs")
        .output()
        .expect("Failed to run corofy for b-coroutines-references");

    // Tell cargo to rerun build script of below file changes
    println!("cargo::rerun-if-changed=stackless-coroutine/src/bin/a-runtime/main_async.rs");
    println!(
        "cargo::rerun-if-changed=stackless-coroutine/src/bin/b-reactor-executor/main_async.rs"
    );
    println!(
        "cargo::rerun-if-changed=stackless-coroutine/src/bin/a-coroutines-variables/main_async.rs"
    );
    println!(
        "cargo::rerun-if-changed=stackless-coroutine/src/bin/b-coroutines-references/main_async.rs"
    );
}
