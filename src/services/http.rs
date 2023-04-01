
use std::sync::Arc;
use std::net::SocketAddr;

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

const HTTP_SERVICE_TYPE: ServiceType = ServiceType::HTTP;
pub struct Http {}

impl Service for Http {
    /// starts a TcpListener that handles requests asynchronously
    fn start(scheduler: Arc<SafeMutex<Scheduler>>, port: u16) {
        // Construct our SocketAddr to listen on...
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let server = Server::http(addr.to_string()).unwrap();
        tracing::info!("HTTP server listening on {:?}", addr);
        for mut request in server.incoming_requests() {
            println!("{:?}",&request);
            
            relay(request, scheduler.clone());
        }
    }
}

fn relay(mut request: tiny_http::Request, scheduler: Arc<SafeMutex<Scheduler>>) {
    
    tokio::spawn(async move {
        let (_rpc_id, UpstreamConfig {rpc_url, ..}) = scheduler.safe_lock(|s| s.schedule(HTTP_SERVICE_TYPE));
        let mut body = String::from("");
        request.as_reader().read_to_string(&mut body);
        // let json_body: Json = body.parse().unwrap();
        tracing::debug!("BODY: {:?}", &body);
        let rpc_request = reqwest::Client::new().post(rpc_url).body(body).header("content-type", "application/json");
        tracing::info!("Sending to RPC {:?}", rpc_request);
        let res = rpc_request.send().await;
        match res {
            Ok(data) => {
                match data.text().await {
                    Ok(text) => {
                        if let Err(e) = request.respond(tiny_http::Response::from_string(text)) {
                            tracing::error!("Failed to respond to client: {:?}", e)
                        }
                    },
                    Err(e) => tracing::error!("Failed to parse body text from RPC: {:?}", e)
                }
                
            },
            Err(e) => tracing::error!("Failed to receive response from RPC: {:?}", e)
        }  
    });
}

