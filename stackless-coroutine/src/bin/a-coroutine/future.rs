#![allow(unused)]

/// Represents some operation that will complete in the future
/// and return a value of type `Future::Output`.
pub trait Future {
    type Output;
    fn poll(&mut self) -> PollState<Self::Output>;
}

/// PollState is an enum that represents the state of a future.
/// It is either Ready or NotReady. The value returned when ready is of type T
pub enum PollState<T> {
    Ready(T),
    NotReady,
}

// Taking inspiration from tokio, we create a `join_all` function
// that takes a collection of futures and drives them all to completion.
pub fn join_all<F: Future>(futures: Vec<F>) -> JoinAll<F> {
    // initialse all futures with a boolean flag of false to indicate they
    // are not complete / not resolved.
    let futures = futures.into_iter().map(|f| (false, f)).collect();

    JoinAll {
        futures,
        finished_count: 0,
    }
}

pub struct JoinAll<F: Future> {
    futures: Vec<(bool, F)>,
    finished_count: usize,
}

// The JoinAll itself is a future and can be polled to completion
impl<F: Future> Future for JoinAll<F> {
    type Output = Vec<<F as Future>::Output>;

    fn poll(&mut self) -> PollState<Self::Output> {
        // store resolved values from all futures and return them
        // when all futures are all resolved.
        let mut resolved_values = vec![];

        for (finished, future) in self.futures.iter_mut() {
            if *finished {
                // don't poll completed future
                continue;
            }

            match future.poll() {
                PollState::NotReady => continue,
                PollState::Ready(value) => {
                    // mark future as resolved
                    *finished = true;
                    self.finished_count += 1;
                    resolved_values.push(value);
                }
            }
        }

        // if all futures are resolved, return Ready
        if self.finished_count == self.futures.len() {
            PollState::Ready(resolved_values)
        } else {
            PollState::NotReady
        }
    }
}
