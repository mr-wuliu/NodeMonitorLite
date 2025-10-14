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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:5051".parse()?;
    println!("server start at {}", addr);
    tonic::transport::Server::builder()
        .add_service(MachineServiceServer::new(MonitorSvc))
        .serve(addr)
        .await?;
    Ok(())
}