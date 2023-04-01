pub mod http;
pub mod websocket;

use std::sync::Arc;

use crate::types::SafeMutex;
use crate::scheduler::Scheduler;


pub trait Service {
    fn start(scheduler: Arc<SafeMutex<Scheduler>>, port: u16);
}