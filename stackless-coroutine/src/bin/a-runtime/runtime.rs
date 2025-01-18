use crate::Future;

pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Runtime
    }
    pub fn block_on(&mut self, future: impl Future) {
        todo!()
    }
}
