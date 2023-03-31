

use std::error::Error;
use std::sync::Arc;
use std::{collections::HashMap, hash::BuildHasherDefault};
use std::str::FromStr;

use http::Request;
use hyper::Body;
use nohash_hasher::NoHashHasher;
use reqwest::Url;
use async_channel::{Sender, Receiver, unbounded};
// use std::sync::mpsc::{Sender, Receiver, channel};
use tokio::io::{BufReader, AsyncWriteExt,AsyncBufReadExt};
use tokio::net::TcpStream;

use crate::config::{UpstreamConfig, Config};
use crate::types::{Message, SafeMutex};
use crate::downstream::{Service, http::Http};

pub type NoHasher = BuildHasherDefault<NoHashHasher<u32>>;
#[derive(Clone)]
pub struct Manager {
    upstreams_rpcs: HashMap<u32, UpstreamConfig, NoHasher>,
    tracker: u32,
}

impl Manager {
    pub async fn start(config: Config) {
        let (downstream_tx, downstream_rx) = unbounded();
        // let (tx, rx) = unbounded();
        // start upstreams
        tracing::info!("Initializing upstreams..");
        let upstreams_rpcs = Self::init_upstreams(config.upstreams);

        // start http listender
        tracing::info!("Starting HTTP listener..");
        Self::start_http_listener(downstream_tx).await.unwrap();

        let manager = Self {
            upstreams_rpcs, 
            tracker: 0,
        };
        // start scheduler
        tracing::info!("Starting scheduler..");
        Self::start_scheduler(
            Arc::new(SafeMutex::new(manager)),
            downstream_rx
        ).await;
    }

    /// starts a single http listener that will relay messages to this component
    async fn start_http_listener(downstream_tx: Sender<Message>) -> Result<(), Box<dyn Error>> {
        Http::start(0, downstream_tx)
    }

    /// should be used to spin up downstream websocket connections in the future
    fn _start_websocket_listener() {}

    /// start upstream 
    fn init_upstreams(upstream_config: Vec<UpstreamConfig>) -> HashMap<u32, UpstreamConfig, NoHasher> {
        let mut upstream_map = HashMap::with_hasher(BuildHasherDefault::default());
        upstream_config.iter().enumerate().for_each(|(i, config)| {
            upstream_map.insert(i as u32, config.clone());
        });
        upstream_map
    }

    pub async fn start_scheduler(self_: Arc<SafeMutex<Self>>, mut downstream_rx: Receiver<Message>) {
        loop {
            println!("LOOP1");
            let msg_opt = downstream_rx.recv().await;
            tracing::info!("Up: {:?}", &msg_opt);
            match msg_opt {
                Ok(msg) => {
                    let (_upstream_id, upstream) = self_.safe_lock(|s| s.schedule());
                    Self::send(upstream.rpc_url, msg).await;
                },
                Err(e) => {
                    tracing::error!("Received no msg from downstream: {:?}", e);
                    break;
                }
            }
            println!("LOOP2");
        }
    }

    fn schedule(&mut self) -> (u32, UpstreamConfig) {
        let upstream_count = self.upstreams_rpcs.keys().count() as u32;
        self.tracker += 1;
        if self.tracker > upstream_count - 1 {
            self.tracker = 0;
        }

        (self.tracker, self.upstreams_rpcs.get(&self.tracker).unwrap().clone())
    }

    async fn send(url: String, mut msg: Message) {
        tokio::spawn(async move{
            let mut body = String::from("");
            msg.request.as_reader().read_to_string(&mut body);
            // let mut rpc_request = reqwest::Request::new(reqwest::Method::GET, Url::from_str(&url).unwrap());
            // let mut rpc_body = rpc_request.body_mut();
            // let body = reqwest::Body::from(body);
            // rpc_body = &mut Some(body);
            println!("BODY: {:?}", &body);
            let rpc_request = reqwest::Client::new().post(url).body(body);
            tracing::info!("Sending to RPC {:?}", rpc_request);
            let res = rpc_request.send().await;
            match res {
                Ok(data) => {
                    match data.text().await {
                        Ok(text) => {
                            println!("TEXT: {:?}", &text);
                            println!("REQ: {:?}", &msg.request);
                            msg.request.remote_addr();
                            if let Err(e) = msg.request.respond(tiny_http::Response::from_string(text)) {
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
    // async fn send(url: String, msg: Message) {
    //     tokio::spawn(async move{
    //         // let request = http::Request::try_from(msg.payload.to_vec());
    //         let rpc_stream = TcpStream::connect(&url).await;
    //         match rpc_stream {
    //             Ok(mut stream) => {
    //                 let _ = stream.write(&msg.payload).await;
    //                 tracing::info!("Sending message to RPC: {:?}\n Message: {:?}", url, &msg.payload);
    //                 let mut reader = BufReader::new(stream);
    //                 loop {
    //                     let mut line = String::new();
    //                     match reader.read_line(&mut line).await {
    //                         Ok(_) => {
    //                             let _ = msg.socket_writer.safe_lock(|w| w.try_write(line.as_bytes()));
                                
    //                         },
    //                         Err(_) => break
    //                     }
    //                 }
    //             },
    //             Err(e) => tracing::error!("Failed to connect to RPC {}\nError: {:?}", url, e)
    //         }
            
    //     });
    // }
}