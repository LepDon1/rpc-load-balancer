pub mod http;

use std::sync::mpsc::Sender;
use async_trait::async_trait;

use crate::types::Message;


#[async_trait]
pub trait Service {
    async fn start(id: u32, relay_channel: Sender<Message>) -> Result<(), Box<dyn std::error::Error>>;
}