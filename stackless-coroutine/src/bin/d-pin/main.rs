#![allow(unused)]
use std::{marker::PhantomPinned, pin::Pin};

#[derive(Default)]
struct Foo {
    a: MaybeSelfRef,
    b: String,
}

#[derive(Default, Debug)]
struct MaybeSelfRef {
    a: usize,
    b: Option<*mut usize>,
    _pin: PhantomPinned,
}

impl MaybeSelfRef {
    /// Pin projection helper method
    ///
    /// This will initialise the type to a
    /// state where it holds a reference to `self`.
    fn init(self: Pin<&mut Self>) {
        unsafe {
            let Self { a, b, .. } = self.get_unchecked_mut();
            // Self-reference start here, where field `b` will hold Option<&mut a>
            *b = Some(a);
        }
    }

    // structural projection, as b is pinned for as long as T is pinned
    fn b(self: Pin<&mut Self>) -> Option<&mut usize> {
        unsafe { self.get_unchecked_mut().b.map(|b| &mut *b) }
    }
}

fn main() {
    // Pin to heap and initialise
    let mut x = Box::pin(MaybeSelfRef::default());

    // Pin<Box<T>> -> Pin<&mut T>
    //  P: Deref & Box: Deref
    //  Pin::as_mut() -> &mut T
    //
    //  So Pin::as_mut is a pin projection that uses structural pinning,
    //  whereby returned Pin<&mut T> is pinned so long as Pin<self> is pinned.
    let pinned_value = x.as_mut();
    pinned_value.init();
    println!("{}", x.as_ref().a);
    let mut x = x;
    // let a = x.as_mut();
    let b = unsafe { x.get_unchecked_mut() };
    *x.as_mut().b().unwrap() = 10;
    println!("{}", x.as_ref().a);
}
