
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub http: Vec<UpstreamConfig>,
    pub websockets: Vec<UpstreamConfig>
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpstreamConfig {
    pub rpc_url: String,
    pub requests_per_min_limit: u32,
}

