use std::sync::OnceLock;

use common::pb::machine_service_client::MachineServiceClient;
use tonic::{transport::Channel, Status};


static GRPC_CLIENT: OnceLock<tokio::sync::Mutex<MachineServiceClient<Channel>>> = OnceLock::new();

pub async fn init_grpc_client(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("init grpc client: {}", addr);
    let channel = Channel::from_shared(addr.to_string())?;
    let client = MachineServiceClient::connect(channel.clone()).await?;
    GRPC_CLIENT
        .set(tokio::sync::Mutex::new(client))
        .map_err(|_| "GRPC client already initialized")?;
    Ok(())
}

pub async fn get_client_clone() -> Result<MachineServiceClient<Channel>, Status> {
    let client = GRPC_CLIENT.get().unwrap().lock().await;
    Ok(client.clone())
}
