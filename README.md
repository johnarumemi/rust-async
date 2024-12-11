# Description

Some experiments with concurrent programming created while working on the `Asynchronous
Programming in Rust (Packt) Book`. When considering wether to place some code here or in
[rust-async-snippets][1], larger projects should be placed here and smaller examples
that explore a very minute / specific concept should be placed in
[rust-async-snippets][1].

## Related Projects

1. [rust-async-snippets (private repo)][1]

    These hold smaller snippets of code created while working through the Async book.

2. [rust-async-utils    (private repo)][2]

    In order to test out various pieces of code created in this book and their
    concurrent behaviour, some additional tools where also created. These are placed in
    this repo.

3. [mini-mio (private repo)][3]

    This expands on the `mini-mio` crate that is a workspace member of this repo.

## Packages

#### mini-mio

> 📝 There is a separate and expanded repo for this now found in 
> the [mini-mio private repo][3].

```bash
cargo run -p mini-mio
```

Simple experiment in creating a library that abstracts over a
platforms provided event queue implementation. In this case, the focus is on
`epoll`.  Hence, this will need to be run on a linux distro and won't work on
Mac OSX or Windows. No assembly is used, so it should work on aarch64 or
x86_64 architectures.

Requirements:
- delayserver (found in [rust-async-utils][2])

#### stackfull-coroutine

fibers / green threads implementation. 

#### stackless-coroutine

lazy future based implementation.

--- 

[1]: https://github.com/johnarumemi/rust-async-snippets "Rust Async Snippets"
[2]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"
[3]: https://github.com/johnarumemi/mini-mio "Expanded mini-mio"
