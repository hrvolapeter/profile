use crate::prelude::*;
use crate::scheduler;
use crate::scheduler::scheduler_client::SchedulerClient;
use bollard::container::InspectContainerOptions;
use bollard::container::Config;
use bollard::Docker;
use std::time::Duration;
use tokio::stream::StreamExt;
use futures_util::stream::TryStreamExt;
use tokio::task::JoinHandle;
use tokio::time::delay_for;
use tonic::codec::Streaming;
use tonic::transport::Channel;

pub struct Task<'a> {
    id: String,
    client: Client,
    docker: &'a Docker,
    measure_handle: Option<JoinHandle<BoxResult<()>>>,
}

impl<'a> Task<'a> {
    fn new(id: String, client: Arc<Mutex<SchedulerClient<Channel>>>, docker: &'a Docker) -> Self {
        Self { id, client, docker, measure_handle: None }
    }

    async fn measure(&mut self) -> BoxResult<()> {
        let id = self.id.clone();
        let client = self.client.clone();
        let docker = self.docker.clone();
        self.measure_handle = Some(tokio::spawn(async move {
            loop {
                let options = InspectContainerOptions { size: false };
                let container = docker.inspect_container(&id, Some(options)).await?;
                debug!("Container '{}' state: '{}'", id, container.state.status);
                if !container.state.running {
                    debug!("Exiting profiling for '{}'", id);
                    client.finish_task(scheduler::FinishTaskRequest {
                        machine_id: MachineId::get().to_string(),
                        task_id: id.clone(),
                    });
                    break;
                }
                let pid = container.state.pid.try_into()?;
                let (mut sender, receiver) = mpsc::channel(10);
                let client = client.clone();
                let id = id.clone();
                let h = tokio::spawn(async move {
                    let profiles =
                        measure::run(Some(vec![pid]), None, Some(receiver)).await?;
                    Self::submit_profile(id, client, profiles).await
                });
                delay_for(Duration::from_secs(30)).await;
                sender.send(()).await?;
                h.await??;
            }
            Ok(())
        }));

        Ok(())
    }

    async fn submit_profile(
        task_id: String,
        client: Arc<Mutex<SchedulerClient<Channel>>>,
        profiles: Vec<measure::ApplicationProfile>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut client = client.lock().await;
        let profiles: Vec<_> = profiles
            .into_iter()
            .map(|x| scheduler::StreamTaskProfilesRequest {
                machine_id: MachineId::get().to_string(),
                task_id: task_id.clone(),
                profile: Some(x.into()),
            })
            .collect();
        client.stream_task_profiles(futures::stream::iter(profiles)).await?;
        Ok(())
    }
}

pub struct TaskRunner<'a> {
    tasks: Vec<Task<'a>>,
    docker: Docker,
}
impl<'a> TaskRunner<'a> {
    pub fn new() -> Self {
        Self { tasks: vec![], docker: Docker::connect_with_local_defaults().unwrap() }
    }
    pub async fn process_tasks(
        &'a mut self,
        client: Client,
        mut tasks: Streaming<scheduler::SubscribeTasksReply>,
    ) -> BoxResult<()> {
        trace!("Processing tasks");
        while let Some(x) = tasks.next().await {
            let task = x?.task.unwrap();
            debug!("Task received '{}'", &task.id);
            use bollard::image::CreateImageOptions;
            use bollard::container::CreateContainerOptions;
            use bollard::container::StartContainerOptions;

            let options = Some(CreateImageOptions{
                from_image: &task.image[..],
                ..Default::default()
              });

            self.docker.create_image(options, None, None).try_collect::<Vec<_>>().await.unwrap();
            let options = Some(CreateContainerOptions { name: task.id.clone() });

            let config = Config {
                image: Some(task.image),
                cmd: task.cmd.map(|x| vec![x]),
                ..Default::default()
            };
            self.docker.create_container(options, config).await?;
            self.docker.start_container(&task.id[..], None::<StartContainerOptions<String>>).await?;
            let mut task = Task::new(task.id.clone(), client.clone(), &self.docker);
            task.measure().await?;
            self.tasks.push(task);
        }
        Ok(())
    }
}
