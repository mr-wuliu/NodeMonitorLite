use clap::Parser;
mod net;
mod task;
mod sampling;
mod config;

use std::fs;
use config::AppConfig;
use crate::task::TaskManager;

#[derive(Parser, Debug)]
#[command(name = "nml", version, about = "Node Monitor Lite")]
struct Args {
    #[arg(long, value_name = "FILE", default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.config)
    .expect("Failed to read config file");
    let mut cfg: AppConfig = toml::from_str(&content)
        .expect("Failed to parse config");

    // initialize grpc client
    net::init_grpc_client(&cfg.client.remote_url).await.unwrap();

    // machine register task
    if cfg.client.uuid.is_none() {
        if let Ok(uuid) = task::machine_register_task(None).await {
            cfg.client.uuid = Some(uuid.unwrap());
            let new_toml = toml::to_string_pretty(&cfg).expect("toml serialize failed");
            std::fs::write(&args.config, new_toml).expect("write config failed");
        } else {
            println!("register machine failed");
            return;
        }
    } else {
        task::machine_register_task(cfg.client.uuid.clone()).await.unwrap();
    }
    
    if cfg.client.uuid.is_none() {
        println!("uuid is not set, please set it in config.toml");
        return;
    }

    let mut manager = TaskManager::new();
    task::start_all_tasks(cfg.client.uuid.clone().unwrap(), &mut manager).await.unwrap();

    loop {
    }
}