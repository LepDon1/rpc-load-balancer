
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

pub struct Scheduler {
    http: HashMap<u32, UpstreamConfig, NoHasher>,
    websockets: HashMap<u32, UpstreamConfig, NoHasher>,
    tracker: u32,
}

impl Scheduler {
    pub fn new(config: Config) -> Arc<SafeMutex<Self>> {
        let http = Self::init_map(config.http);
        let websockets = Self::init_map(config.websockets);
        Arc::new(SafeMutex::new(
            Self {
                http,
                websockets,
                tracker: 0
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


    pub fn schedule(&mut self, service_type: ServiceType) -> (u32, UpstreamConfig) {
        let map = match service_type {
            ServiceType::HTTP => self.http.clone(),
            ServiceType::Websocket => self.websockets.clone()
        };

        let count = map.keys().count() as u32;
        self.tracker += 1;
        if self.tracker > count - 1 {
            self.tracker = 0;
        }

        (self.tracker, map.get(&self.tracker).unwrap().clone())
    }
}