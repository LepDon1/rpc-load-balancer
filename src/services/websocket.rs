
use std::sync::Arc;
use std::net::SocketAddr;
use core::fmt::write;
use std::iter::Fuse;
use simple_websockets::{Event, Responder, Message};
use std::collections::HashMap;

use futures_util::{future, pin_mut, StreamExt, SinkExt, FutureExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::{
    BufReader,
    AsyncRead,
    AsyncBufReadExt,
};
use tokio_tungstenite::connect_async;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::net::{TcpListener, TcpStream};
use async_trait::async_trait;
use tiny_http::{Server, Response};

use super::Service;
use crate::config::UpstreamConfig;
use crate::scheduler::{Scheduler, ServiceType};
use crate::types::SafeMutex;

const WS_SERVICE_TYPE: ServiceType = ServiceType::Websocket;

pub enum Notify {
    Message(Message),
    Disconnect
}

#[derive(Clone)]
pub struct Channel {
    pub client_id: u64,
    pub responder: Responder,
    pub tx: Sender<Notify>
}


impl Channel {
    pub async fn connect(self_: Arc<SafeMutex<Channel>>, mut rx: Receiver<Notify>, upstream_rpc: UpstreamConfig) -> Result<(), Box<dyn std::error::Error>> {
        // connect to rpc and start loop
        let (rpc_stream, _) = connect_async(upstream_rpc.rpc_url).await?;
        let (write, read) = rpc_stream.split();
        
        let rpc_read_fut = read.for_each(|rpc_msg_res| async {
            match rpc_msg_res {
                Ok(rpc_msg) => {
                    self_.safe_lock(|s| s.responder.send(Message::Text(rpc_msg.to_string())));
                }
                Err(e) => {tracing::error!("Failed to recv rpc message: {:?}", e)}
            }
            
        });
        pin_mut!(rpc_read_fut, write);

        tokio::select! {
            _ = rpc_read_fut => {},
            downstream_msg = rx.recv().fuse() => {
                match downstream_msg {
                    Some(Notify::Message(msg)) => {
                        let _ = match msg {
                            Message::Text(text) => write.send(tokio_tungstenite::tungstenite::Message::Text(text)).await,
                            Message::Binary(bin) => write.send(tokio_tungstenite::tungstenite::Message::Binary(bin)).await
                        };
                        
                    },
                    Some(Notify::Disconnect) => {return Ok(());},
                    None => {}

                }
            }
        }

        Ok(())
        
    }
}
pub struct Websocket {
    channels: HashMap<u64, Arc<SafeMutex<Channel>>>
}

#[async_trait]
impl Service for Websocket {
    /// starts a TcpListener that handles requests asynchronously
    fn start(scheduler: Arc<SafeMutex<Scheduler>>, port: u16) {
        // listen for WebSockets on port 8080:
        let event_hub = simple_websockets::launch(port)
        .expect("failed to listen on port 8001");
        // map between client ids and the client's `Responder`:
        let channels: HashMap<u64, Arc<SafeMutex<Channel>>> = HashMap::new();

        let manager = Arc::new(SafeMutex::new(
            Websocket {
            channels
        }));
        tracing::info!("Websocket server listening port {:?}", port);
        loop {
            match event_hub.poll_event() {
                Event::Connect(client_id, responder) => {
                    println!("A client connected with id #{}", client_id);
                    Self::connect(manager.clone(), client_id, responder, scheduler.clone());
                },
                Event::Disconnect(client_id) => {
                    println!("Client #{} disconnected.", client_id);
                    Self::disconnect(manager.clone(), &client_id)
                },
                Event::Message(client_id, message) => {
                    println!("Received a message from client #{}: {:?}", client_id, message);
                    // retrieve this client's `Responder`:
                    Self::message(manager.clone(), client_id, message)
                },
            }
        }
    }
}

impl Websocket {
    fn connect(self_: Arc<SafeMutex<Self>>, client_id: u64, responder: Responder, scheduler: Arc<SafeMutex<Scheduler>>) {
        // schedule websocket
        let (_rpc_id, upstream_rpc) = scheduler.safe_lock(|s| s.schedule(ServiceType::Websocket));
        let (tx, rx) = channel(100);
        // set up websocket with rpc
        let channel = Arc::new(SafeMutex::new(
            Channel {
            client_id, 
            responder,
            tx
        }));

        self_.safe_lock(|s| s.channels.insert(client_id, channel.clone()));
        tokio::spawn(async move {
            // start task
            match Channel::connect(channel, rx, upstream_rpc.clone()).await {
                // save channel
                Ok(_) => {
                    self_.safe_lock(|s| s.channels.remove(&client_id));
                    tracing::info!("Shutting down WS connection {:?}", upstream_rpc.rpc_url);
                },
                Err(_) => {
                    self_.safe_lock(|s| s.channels.remove(&client_id));
                    tracing::error!("Failed to connect to upstream WS {:?}", upstream_rpc.rpc_url);
                }
            }
        });
        
        

    }

    fn disconnect(self_: Arc<SafeMutex<Self>>, client_id: &u64) {
        if let Some(channel) = self_.safe_lock(|s| s.channels.remove(client_id)) {
            tokio::spawn(async move {
                let tx = channel.safe_lock(|c| c.tx.clone());
                let _ = tx.send(Notify::Disconnect).await;
            });
        }
    }

    fn message(self_: Arc<SafeMutex<Self>>, client_id: u64, message: Message) {
        let self_clone = self_.clone();
        if let Some(channel) = self_.safe_lock(|s| s.channels.get(&client_id).map(|v| v.clone())) {
            tokio::spawn(async move {
                let tx = channel.safe_lock(|c| c.tx.clone());
                if let Err(_e) = tx.send(Notify::Message(message)).await {
                    self_clone.safe_lock(|s| s.channels.remove(&client_id));
                }
            });
        }
    }
}

// fn relay(mut request: Message, scheduler: Arc<SafeMutex<Scheduler>>) {
    
//     tokio::spawn(async move {
//         let (_rpc_id, UpstreamConfig {rpc_url, ..}) = scheduler.safe_lock(|s| s.schedule(WS_SERVICE_TYPE));
//         let mut body = String::from("");
//         request.as_reader().read_to_string(&mut body);
//         // let json_body: Json = body.parse().unwrap();
//         tracing::debug!("BODY: {:?}", &body);
//         let rpc_request = reqwest::Client::new().post(rpc_url).body(body).header("content-type", "application/json");
//         tracing::info!("Sending to RPC {:?}", rpc_request);
//         let res = rpc_request.send().await;
//         match res {
//             Ok(data) => {
//                 match data.text().await {
//                     Ok(text) => {
//                         if let Err(e) = request.respond(tiny_http::Response::from_string(text)) {
//                             tracing::error!("Failed to respond to client: {:?}", e)
//                         }
//                     },
//                     Err(e) => tracing::error!("Failed to parse body text from RPC: {:?}", e)
//                 }
                
//             },
//             Err(e) => tracing::error!("Failed to receive response from RPC: {:?}", e)
//         }  
//     });
// }

