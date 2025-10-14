use tokio::net::TcpListener;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tokio::io::{BufReader, AsyncWriteExt, AsyncBufReadExt};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let grpc_addr = "0.0.0.0:5051".parse()?;
    println!("server start at {}", grpc_addr);
    let grpc_svc = MachineServiceServer::new(MonitorSvc);

    // TCO CLI service
    let tcp_addr: std::net::SocketAddr = "0.0.0.0:5052".parse()?;
    let tcp_listener = TcpListener::bind(tcp_addr).await?;
    println!("CLI TCP listening on 127.0.0.1:5052");

    tokio::try_join!(
        async move {
            Server::builder()
                .add_service(grpc_svc)
                .serve(grpc_addr)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        async move {
            loop {
                let (socket, _) = tcp_listener.accept().await?;
                tokio::spawn(async move {
                    let (reader, mut writer) = socket.into_split();
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();

                    while reader.read_line(&mut line).await.unwrap() > 0 {
                        let cmd = line.trim();
                        let reply = match cmd {
                            "status" => "daemon running normally\n".to_string(),
                            "reload" => {
                                println!("reload requested");
                                "reloaded\n".to_string()
                            },
                            _ => "unknown command\n".to_string(),
                        };
                        writer.write_all(reply.as_bytes()).await.unwrap();
                        line.clear();
                    }
                });
            }
            #[allow(unreachable_code)]
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }
    )?;
    Ok(())
}