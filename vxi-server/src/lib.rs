#![no_std]

use arrayvec::{Array, ArrayVec};
use managed::ManagedSlice;
use smoltcp::socket::{
    AnySocket, Socket, SocketHandle, SocketRef, SocketSet, SocketSetItem,
};
use core::cell::RefCell;

pub mod hislip;
pub mod vxi11;
pub mod socket;

pub mod device;

pub struct SocketPool<A: Array> {
    available: RefCell<ArrayVec<A>>
}

impl<A> SocketPool<A>
where
    A: Array<Item = SocketHandle>,
{
    pub fn new() -> Self {
        let sm = Self {
            available: RefCell::new(ArrayVec::new())
        };
        sm
    }

    pub fn add_socket(&mut self, handle: SocketHandle) -> SocketHandle {
        self.available.borrow_mut().push(handle);
        handle
    }

    /// Get a socket from pool
    pub fn get_available_socket(&mut self) -> Option<SocketHandle> {
        self.available.borrow_mut().pop_at(0)
    }

    pub fn return_socket(&self, handle: SocketHandle) {
        self.available.borrow_mut().push(handle);
    }
}



