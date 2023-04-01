
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
        let client = reqwest::Client::new();
        let scheduler = scheduler.safe_lock(|s| s.clone());
        for request in server.incoming_requests() {
            
            relay(client.clone(), request, scheduler.clone());
        }
    }
}

fn relay(client: reqwest::Client, mut request: tiny_http::Request, mut scheduler: Scheduler) {
    let (_rpc_id, UpstreamConfig {rpc_url, ..}) = scheduler.schedule_http();
    let timer = std::time::Instant::now();
    tokio::spawn(async move {
        let mut body = String::from("");
        request.as_reader().read_to_string(&mut body);
        // let json_body: Json = body.parse().unwrap();
        tracing::debug!("BODY: {:?}", &body);
        let rpc_request = client.post(rpc_url).body(body).header("content-type", "application/json");
        // tracing::info!("Sending to RPC {:?}", rpc_request);
        tracing::info!("Elapsed time to UP: {:?}", timer.elapsed());
        let res = rpc_request.send().await;
        let timer = std::time::Instant::now();
        match res {
            Ok(data) => {
                match data.text().await {
                    Ok(text) => {
                        if let Err(e) = request.respond(tiny_http::Response::from_string(text)) {
                            tracing::error!("Failed to respond to client: {:?}", e)
                        }
                        tracing::info!("Elapsed time to DOWN: {:?}", timer.elapsed());
                    },
                    Err(e) => tracing::error!("Failed to parse body text from RPC: {:?}", e)
                }
                
            },
            Err(e) => tracing::error!("Failed to receive response from RPC: {:?}", e)
        }  
    });
}

