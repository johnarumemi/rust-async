//! Future related code

pub trait Future {
    type Output;
    fn poll(&mut self) -> PollState<Self::Output>;
}

pub enum PollState<T> {
    Ready(T),
    NotReady,
}

pub struct JoinAll<F: Future> {
    futures: Vec<(bool, F)>,
    finished_count: usize,
}

impl<F> From<Vec<F>> for JoinAll<F>
where
    F: Future,
{
    fn from(futures: Vec<F>) -> Self {
        JoinAll {
            futures: futures.into_iter().map(|f| (false, f)).collect(),
            finished_count: 0,
        }
    }
}

#[allow(dead_code)]
pub fn join_all<F: Future>(futures: Vec<F>) -> JoinAll<F> {
    futures.into()
}

impl<F: Future> Future for JoinAll<F> {
    type Output = String;
    fn poll(&mut self) -> PollState<Self::Output> {
        for (finished, future) in self.futures.iter_mut() {
            if *finished {
                // skip finished futures
                continue;
            }

            match future.poll() {
                // future is ready, mark as finished
                PollState::Ready(_) => {
                    *finished = true;
                    self.finished_count += 1;
                }
                // continue to poll next future
                PollState::NotReady => continue,
            }
        }

        if self.finished_count == self.futures.len() {
            // All futures are
            //
            // resolve with an empty string for now.
            // `corofy` will only work with futures that resolve to a String
            PollState::Ready(String::new())
        } else {
            PollState::NotReady
        }
    }
}
