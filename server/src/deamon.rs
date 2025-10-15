use tokio::net::TcpListener;
use tokio::io::{BufReader, AsyncWriteExt, AsyncBufReadExt};
use tokio::sync::watch;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use common::pb::{
    machine_service_server::{MachineService, MachineServiceServer}, MachineClientId, MachineInfo, MachineDynamicInfo
};
use uuid::Uuid;

#[derive(Default)]
struct MonitorSvc;

#[tonic::async_trait]
impl MachineService for MonitorSvc {
    async fn register_machine(
        &self,
        request: Request<MachineInfo>,
    ) -> Result<Response<MachineClientId>, Status> {
        println!("register machine: {:?}", request.get_ref());

        let uuid = request.get_ref().uuid.clone();
        match uuid {
            Some(uuid) => {
                return Ok(Response::new(MachineClientId { uuid: uuid }));
            }
            None => {
                return Ok(Response::new(MachineClientId { uuid: Uuid::new_v4().to_string() }));
            }
        }
    }

    async fn report_dynamic_info(
        &self,
        request: Request<MachineDynamicInfo>,
    ) -> Result<Response<()>, Status> {
        println!("report dynamic info: {:?}", request.get_ref());
        Ok(Response::new(()))
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (shutdown_tx, shutdown_rx) = watch::channel::<bool>(false);

    let grpc_addr = "0.0.0.0:5051".parse()?;
    println!("server start at {}", grpc_addr);
    let grpc_svc = MachineServiceServer::new(MonitorSvc);

    // TCP CLI service
    let tcp_addr: std::net::SocketAddr = "0.0.0.0:5052".parse()?;
    let tcp_listener = TcpListener::bind(tcp_addr).await?;
    println!("CLI TCP listening on {}", tcp_addr);

    // gRPC server with graceful shutdown
    let mut grpc_shutdown_rx = shutdown_rx.clone();
    let grpc_task = async move {
        Server::builder()
            .add_service(grpc_svc)
            .serve_with_shutdown(grpc_addr, async move {
                loop {
                    let changed = grpc_shutdown_rx.changed().await;
                    if changed.is_err() { break; }
                    if *grpc_shutdown_rx.borrow() { break; }
                }
            })
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    };

    // TCP CLI accept loop with shutdown
    let mut cli_shutdown_rx = shutdown_rx.clone();
    let cli_task = async move {
        loop {
            tokio::select! {
                biased;
                _ = cli_shutdown_rx.changed() => {
                    if *cli_shutdown_rx.borrow() { break; }
                }
                accept_res = tcp_listener.accept() => {
                    let (socket, _) = accept_res?;
                    let local_shutdown_tx = shutdown_tx.clone();
                    tokio::spawn(async move {
                        let (reader, mut writer) = socket.into_split();
                        let mut reader = BufReader::new(reader);
                        let mut line = String::new();

                        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                            let cmd = line.trim();
                            let reply = match cmd {
                                "status" => "daemon running normally\n".to_string(),
                                "stop" => {
                                    let _ = local_shutdown_tx.send(true);
                                    "stopping\n".to_string()
                                },
                                _ => "unknown command\n".to_string(),
                            };
                            if writer.write_all(reply.as_bytes()).await.is_err() {
                                break;
                            }
                            line.clear();
                        }
                    });
                }
            }
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    tokio::try_join!(grpc_task, cli_task)?;
    Ok(())
}


