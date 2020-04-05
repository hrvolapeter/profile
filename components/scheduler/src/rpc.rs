use tonic::{Request, Response, Status};

pub use scheduler::scheduler_server::{Scheduler, SchedulerServer};
use scheduler::{RegistrationReply, RegistrationRequest};

pub mod scheduler {
    tonic::include_proto!("scheduler"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct SchedulerService {}

#[tonic::async_trait]
impl Scheduler for SchedulerService {
    async fn register_server(
        &self,
        request: Request<RegistrationRequest>,
    ) -> Result<Response<RegistrationReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = scheduler::RegistrationReply { should_benchmark: true };

        Ok(Response::new(reply))
    }
}
