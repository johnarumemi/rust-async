//! Future related code
use crate::runtime::Waker;

pub enum PollState<T> {
    Ready(T),
    NotReady,
}

pub trait Future {
    type Output;
    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output>;
}
