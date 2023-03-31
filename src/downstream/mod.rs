pub mod http;

use async_channel::Sender;
use async_trait::async_trait;

use crate::types::Message;


#[async_trait]
pub trait Service {
    fn start(id: u32, relay_channel: Sender<Message>) -> Result<(), Box<dyn std::error::Error>>;
}