
use std::sync::Arc;
use std::net::SocketAddr;
use simple_websockets::{Event, Responder, Message};
use tokio::net::unix::pipe::Receiver;
use std::collections::HashMap;

use tokio::io::{
    BufReader,
    AsyncRead,
    AsyncBufReadExt,
};
use tokio::net::{TcpListener, TcpStream};
use async_trait::async_trait;
use tiny_http::{Server, Response};

use super::Service;
use crate::config::UpstreamConfig;
use crate::scheduler::{Scheduler, ServiceType};
use crate::types::SafeMutex;

const WS_SERVICE_TYPE: ServiceType = ServiceType::Websocket;

pub struct Connection {
    client_id: u64,
    responder: Responder,
    shutdown: bool
}
pub struct Websocket {}

#[async_trait]
impl Service for Websocket {
    /// starts a TcpListener that handles requests asynchronously
    fn start(scheduler: Arc<SafeMutex<Scheduler>>, port: u16) {
        // listen for WebSockets on port 8080:
        let event_hub = simple_websockets::launch(port)
        .expect("failed to listen on port 8080");
        // map between client ids and the client's `Responder`:
        let mut clients: HashMap<u64, Responder> = HashMap::new();

        loop {
            match event_hub.poll_event() {
                Event::Connect(client_id, responder) => {
                    println!("A client connected with id #{}", client_id);
                    // add their Responder to our `clients` map:
                    clients.insert(client_id, responder);
                },
                Event::Disconnect(client_id) => {
                    println!("Client #{} disconnected.", client_id);
                    // remove the disconnected client from the clients map:
                    clients.remove(&client_id);
                },
                _ => {},
            }
        }
    }
}

impl Websocket {
    fn connect(client_id: u64, scheduler: Arc<SafeMutex<Scheduler>>) {
        // schedule websocket
        // set up websocket with rpc

    }

    fn disconnect() {}
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

