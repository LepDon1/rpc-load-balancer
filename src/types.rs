use std::{sync::{Arc, Mutex}, net::TcpStream};

use serde::Serialize;
use tokio::{io::BufReader, net::tcp::OwnedWriteHalf};
use tiny_http::Request;

#[derive(Debug)]
pub struct Message {
    // pub socket_writer: Arc<SafeMutex<OwnedWriteHalf>>,
    pub request: Request
}

pub struct SafeMutex<T: ?Sized>(Mutex<T>);

impl<T> SafeMutex<T> {
    pub fn new(v: T) -> Self {
        Self(Mutex::new(v))
    }

    pub fn safe_lock<F, Ret>(&self, func: F) -> Ret
    where 
        F: FnOnce(&mut T) -> Ret
    {
        let mut lock = self.0.lock().unwrap();
        let res = func(&mut *lock);
        drop(lock);
        res
    }
}