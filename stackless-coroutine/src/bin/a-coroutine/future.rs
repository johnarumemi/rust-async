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
