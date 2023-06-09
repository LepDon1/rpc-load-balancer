

//! 1. Set up a listener to handle incoming set up incoming Downstream connections
//! 2. Build RPC connection for each config Upstream connection
//! 3. Downstream connections get relayed to Upstream based on Scheduler
//! 4. scheduler can be extended for optimization
//! 
//! Start with strictly HTTP downtream connections but we can extend later to bridge downstream websockets to upstream HTTP calls so we can optimize polling frequency
//! on this server instead of downstreams setting the polling frequency
//! 
//! 


mod services;
// mod manager;
mod scheduler;
mod config;
mod args;
mod types;
use std::sync::Arc;

use tokio;
use toml;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use config::Config;
use args::Args;
use types::SafeMutex;
use services::Service;
use scheduler::Scheduler;
// use manager::Manager;


fn process_cli_args<'a>() -> Config {
    let args = match Args::from_args() {
        Ok(cfg) => cfg,
        Err(help) => {
            tracing::error!("{}", help);
            panic!("Invalid cli command");
        }
    };
    let config_file = std::fs::read_to_string(args.config_path).unwrap();
    toml::from_str::<Config>(&config_file).unwrap()
}


#[tokio::main]
async fn main() {
    // a builder for `FmtSubscriber`.
    // let subscriber = FmtSubscriber::builder()
    //     // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    //     // will be written to stdout.
    //     .with_max_level(Level::INFO)
    //     // completes the builder.
    //     .finish();
    tracing_subscriber::fmt::init();

    // tracing::subscriber::set_global_default(subscriber)
        // .expect("setting default subscriber failed");

    let config = process_cli_args();
    tracing::info!("Config: {:?}", &config);

    let scheduler = Scheduler::new(config);
    let scheduler_clone = scheduler.clone();
    
    // start Websocket service 
    tokio::spawn(async move {
        services::websocket::Websocket::start(scheduler_clone, 8001);
    });

    // start HTTP service
    services::http::Http::start(scheduler, 8000);

    
}
