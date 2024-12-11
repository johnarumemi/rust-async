# Description

Minitature implementation of mio.


## Requirements

- linux x86_64
- delayserver (see [rust-async-utils][1] private repo)


## Usage

This requires linux x86_64, so will need to make use of the devcontainer with colima
running a x86_64 VM. 

To run the `mini-mio` binary, use the following command:
```bash
cargo run -p mini-mio
```

## Troubleshooting

#### Cannot reach server

To confirm that the delay server is reachable use the following
command:
```bash
curl http://host.docker.internal:<port>/<delay in ms: int>/<messages: string>
```
for example,
```bash
curl http://host.docker.internal:8080/1000/hello-world
```

---

[1]: https://github.com/johnarumemi/rust-async-utils "Rust Async Utils"
