# Description
some simple experiments with concurrent programming.

## Related Projects
- `rust-async-snippets` (private)
- `rust-async-utils`    (private)

## Workspace Members

- `a-stack-swap`: experiment with how to use inline-assembly to swap CPU 
  state and execute on a different thread's stack. This is preliminary work
  on implementing a stackfull-coroutine (fibers and green threads).  

  Requirements:
  - Unix family platform 
  - x86_64 CPU Architecture

  ```bash
  cargo run -p a-stack-swap
  ```

- `mini-mio`: Simple experiment in creating a library that abstracts over a
  platforms provided event queue implementation. In this case, the focus is on
  `epoll`.  Hence, this will need to be run on a linux distro and won't work on
  Mac OSX or Windows. No assembly is used, so it should work on aarch64 or
  x86_64 architectures.

- `stackfull-coroutine`: fibers / green threads implementation. 

- `stackless-coroutine`: lazy future based implementation.

- `delayserver`: utility package that spins up a delay server, used for testing
  out functionality in other packages.
