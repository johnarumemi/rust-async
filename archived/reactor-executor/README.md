# Running 


## a-runtime
First example that uses a tightly coupled reactor executor pattern
```bash
cargo run -p reactor-executor --bin a-runtime
```

## b-reactor-executor
Introduce the use of a Waker to separate out tight coupling of current reactor
and executor implementation. They both know about "Poll" and the executor is
blocked on waiting for new event notifications


```bash
cargo run -p reactor-executor --bin b-reactor-executor
```

```bash
cargo run -p reactor-executor --bin c-reactor-executor
```

```bash
cargo run -p reactor-executor --bin d-reactor-executor
```
