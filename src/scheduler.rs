
use std::{collections::HashMap, hash::BuildHasherDefault};
use std::sync::Arc;
use http::header::SERVER;
use nohash_hasher::NoHashHasher;

use crate::config::{UpstreamConfig, Config};
use crate::types::SafeMutex;

pub type NoHasher = BuildHasherDefault<NoHashHasher<u32>>;


pub enum ServiceType {
    HTTP,
    Websocket
}

#[derive(Clone)]
pub struct Scheduler {
    http: HashMap<u32, UpstreamConfig, NoHasher>,
    websockets: HashMap<u32, UpstreamConfig, NoHasher>,
    http_tracker: u32,
    ws_tracker: u32
}

impl Scheduler {
    pub fn new(config: Config) -> Arc<SafeMutex<Self>> {
        let http = Self::init_map(config.http);
        let websockets = Self::init_map(config.websockets);
        Arc::new(SafeMutex::new(
            Self {
                http,
                websockets,
                http_tracker: 0,
                ws_tracker: 0
            }
        ))
    }

    fn init_map(upstream_config: Vec<UpstreamConfig>) -> HashMap<u32, UpstreamConfig, NoHasher> {
        let mut upstream_map = HashMap::with_hasher(BuildHasherDefault::default());
        upstream_config.iter().enumerate().for_each(|(i, config)| {
            upstream_map.insert(i as u32, config.clone());
        });
        upstream_map
    }


    pub fn schedule_http(&mut self) -> (u32, UpstreamConfig) {
        let count = self.http.keys().count() as u32;
        self.http_tracker += 1;
        if self.http_tracker > count - 1 {
            self.http_tracker = 0;
        }

        (self.http_tracker, self.http.get(&self.http_tracker).unwrap().clone())
    }

    pub fn schedule_ws(&mut self) -> (u32, UpstreamConfig) {
        let count = self.websockets.keys().count() as u32;
        self.ws_tracker += 1;
        if self.ws_tracker > count - 1 {
            self.ws_tracker = 0;
        }

        (self.ws_tracker, self.websockets.get(&self.ws_tracker).unwrap().clone())
    }

}