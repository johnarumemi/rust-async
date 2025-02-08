//! future related code
#![allow(unused)]
use std::pin::Pin;

use crate::runtime::Waker;

/// Represents some operation that will complete in the future
/// and return a value of type `Future::Output`.
pub trait Future {
    type Output;
    // NEW: When we poll a future, we must now supply a Waker
    fn poll(self: Pin<&mut Self>, waker: &Waker) -> PollState<Self::Output>;
}

/// PollState is an enum that represents the state of a future.
/// It is either Ready or NotReady. The value returned when ready is of type T
pub enum PollState<T> {
    Ready(T),
    NotReady,
}
