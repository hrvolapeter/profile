use machine_id::MachineId;
use scheduler::scheduler_client::SchedulerClient;
use scheduler::RegistrationRequest;
use tonic::transport::Channel;

pub mod scheduler {
    tonic::include_proto!("scheduler");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SchedulerClient::connect("http://[::1]:50051").await?;
    register(&mut client).await?;
    Ok(())
}

async fn register(client: &mut SchedulerClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {
    let request =
        tonic::Request::new(RegistrationRequest { machine_id: MachineId::get().to_string() });

    let response = client.register_server(request).await?.into_inner();
    if response.should_benchmark {
        benchmark::run().await?;
    }
    Ok(())
}
