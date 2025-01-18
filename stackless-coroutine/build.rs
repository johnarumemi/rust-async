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
        .expect("Failed to run corofy");

    // Tell cargo to rerun build script of below file changes
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=stackless-coroutine/src/bin/a-runtime/main_async.rs");
}
