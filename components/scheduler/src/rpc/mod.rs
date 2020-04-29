use crate::import::*;
use crate::scheduler;
use futures::channel::mpsc;
use futures::{FutureExt, StreamExt};
use log::debug;
use tonic::{Request, Response, Status};

pub use proto::scheduler_server::{Scheduler, SchedulerServer};
use proto::{RegistrationReply, RegistrationRequest};

pub mod proto {
    tonic::include_proto!("scheduler"); // The string specified here must match the proto package name
}
type SchedulerObj = Arc<Mutex<scheduler::Scheduler>>;

pub struct SchedulerService {
    scheduler: SchedulerObj,
}

impl SchedulerService {
    pub fn new(scheduler: SchedulerObj) -> Self {
        Self { scheduler }
    }
}

#[tonic::async_trait]
impl Scheduler for SchedulerService {
    type SubscribeTasksStream = mpsc::Receiver<Result<proto::SubscribeTasksReply, tonic::Status>>;

    async fn register_server(
        &self,
        request: Request<RegistrationRequest>,
    ) -> Result<Response<RegistrationReply>, Status> {
        let request = request.into_inner();
        debug!("Registering server id: '{}'", request.machine_id);
        self.scheduler
            .lock()
            .await
            .insert_server(scheduler::Server::new(
                Uuid::parse_str(&request.machine_id).unwrap(),
                request.hostname.clone(),
                None,
            ))
            .await;

        let reply = proto::RegistrationReply { should_benchmark: true };

        Ok(Response::new(reply))
    }

    async fn submit_benchmark(
        &self,
        request: Request<proto::BenchmarkSubmitRequest>,
    ) -> Result<Response<proto::BenchmarkSubmitReply>, Status> {
        let request = request.into_inner();
        debug!("Received benchmark from server id: '{}'", request.machine_id);
        let mut sch = self.scheduler.lock().await;
        let server = sch.get_server(&Uuid::from_str(&request.machine_id).unwrap()).unwrap();
        debug!("Registering server with profile: '{:?}'", server);
        server.set_profile(Some(request.profile.unwrap().into()));
        sch.schedule().await;
        let reply = proto::BenchmarkSubmitReply {};

        Ok(Response::new(reply))
    }

    async fn subscribe_tasks(
        &self,
        request: Request<proto::SubscribeTasksRequest>,
    ) -> Result<Response<Self::SubscribeTasksStream>, tonic::Status> {
        let (sched_tx, sched_rx) = mpsc::channel(10);

        let request = request.into_inner();
        self.scheduler
            .lock()
            .await
            .subscribe_server(Uuid::parse_str(&request.machine_id).unwrap(), sched_tx);

        let (tx, rx) = mpsc::channel(10);

        tokio::task::spawn(
            sched_rx
                .map(|x| {
                    let task = Some(x.task.into());
                    let state: proto::subscribe_tasks_reply::State = x.state.into();
                    let res = proto::SubscribeTasksReply { task, state: state as i32 };
                    Ok(Ok(res))
                })
                .forward(tx)
                .map(|result| {
                    if let Err(e) = result {
                        error!("task send error: {}", e);
                    }
                }),
        );

        Ok(Response::new(rx))
    }

    async fn stream_task_profiles(
        &self,
        request: Request<tonic::Streaming<proto::StreamTaskProfilesRequest>>,
    ) -> Result<Response<proto::StreamTaskProfilesReply>, Status> {
        let mut stream = request.into_inner();
        while let Some(request) = stream.next().await {
            let request = request?;
            let mut sched = self.scheduler.lock().await;
            let task = sched.get_task(&Uuid::from_str(&request.task_id).unwrap()).unwrap();
            trace!("Received profile for '{}', '{:?}'", &request.task_id, &request.profile);
            task.insert_profile(
                Uuid::from_str(&request.machine_id).unwrap(),
                request.profile.unwrap().into(),
            );
            sched.schedule().await;
        }
        Ok(Response::new(proto::StreamTaskProfilesReply {}))
    }

    async fn finish_task(
        &self,
        request: Request<proto::FinishTaskRequest>
    ) -> Result<Response<proto::FinishTaskReply>, Status> {
        let request = request.into_inner();

        let mut sched = self.scheduler.lock().await;
        sched.get_task(&Uuid::from_str(&request.task_id).unwrap()).unwrap().set_schedulable(false);
        sched.schedule().await;
        Ok(Response::new(proto::FinishTaskReply {}))
    }
}

impl Into<scheduler::ResourceProfile> for proto::Profile {
    fn into(self) -> scheduler::ResourceProfile {
        scheduler::ResourceProfile {
            ipc: Decimal::new(self.instructions as i64, 0).checked_div(Decimal::new(self.cycles as i64, 0)).unwrap_or(Decimal::new(0,0)),
            disk: self.vfs_read + self.vfs_write,
            memory: self.memory,
            network: self.tcp_send_bytes + self.tcp_recv_bytes,
        }
    }
}

impl From<scheduler::Task<scheduler::ResourceProfile>> for proto::subscribe_tasks_reply::Task {
    fn from(task: scheduler::Task<scheduler::ResourceProfile>) -> Self {
        Self {
            id: task.id().to_string(),
            image: task.image().clone(),
            is_profiled: task.request().is_none(),
            cmd: task.cmd().clone(),
        }
    }
}

impl From<scheduler::State> for proto::subscribe_tasks_reply::State {
    fn from(state: scheduler::State) -> Self {
        match state {
            scheduler::State::Run => Self::Run,
            scheduler::State::Remove => Self::Remove,
        }
    }
}
