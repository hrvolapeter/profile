#![deny(warnings)]

mod task;
mod scheduler {
    tonic::include_proto!("scheduler");
}
mod prelude {
    use crate::scheduler::scheduler_client::SchedulerClient;
    use std::error::Error;
    use tonic::transport::Channel;

    pub(crate) use {
        log::debug, log::trace, machine_id::MachineId, std::convert::TryInto, std::sync::Arc,
        tokio::sync::mpsc, tokio::sync::Mutex,
    };
    pub type Client = Arc<Mutex<SchedulerClient<Channel>>>;
    pub type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
}

use crate::prelude::*;
use crate::scheduler::scheduler_client::SchedulerClient;
use fern::colors::ColoredLevelConfig;
use std::cmp::max;
use tonic::codec::Streaming;

#[tokio::main]
async fn main() -> BoxResult<()> {
    setup_logger()?;
    let client = Arc::new(Mutex::new(SchedulerClient::connect("http://[::1]:50051").await?));
    let tasks = subscribe_tasks(client.clone()).await?;
    let registration = register(client.clone());

    let mut task_runner = task::TaskRunner::new();
    let tasks = task_runner.process_tasks(client, tasks);
    futures::try_join!(registration, tasks)?;
    Ok(())
}

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .debug(fern::colors::Color::Green)
        .trace(fern::colors::Color::Blue);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

async fn register(client: Client) -> BoxResult<()> {
    let mut client = client.lock().await;
    let request = tonic::Request::new(scheduler::RegistrationRequest {
        machine_id: MachineId::get().to_string(),
        hostname: hostname::get()?.into_string().unwrap(),
    });

    let response = client.register_server(request).await?.into_inner();
    if response.should_benchmark {
        let profiles = benchmark::run().await?;
        let mut profile = get_maximum(profiles);
        profile.memory =  sys_info::mem_info()?.total;
        let request = tonic::Request::new(scheduler::BenchmarkSubmitRequest {
            machine_id: MachineId::get().to_string(),
            profile: Some(profile.into()),
        });
        client.submit_benchmark(request).await?;
    }
    Ok(())
}

async fn subscribe_tasks(client: Client) -> BoxResult<Streaming<scheduler::SubscribeTasksReply>> {
    let mut client = client.lock().await;
    let request = tonic::Request::new(scheduler::SubscribeTasksRequest {
        machine_id: MachineId::get().to_string(),
    });

    let response = client.subscribe_tasks(request).await?.into_inner();
    Ok(response)
}

fn get_maximum(profiles: Vec<measure::ApplicationProfile>) -> measure::ApplicationProfile {
    profiles.into_iter().fold(Default::default(), |x, y| measure::ApplicationProfile {
        cache_misses: max(x.cache_misses, y.cache_misses),
        cache_references: max(x.cache_references, y.cache_references),
        vfs_write: max(x.vfs_write, y.vfs_write),
        vfs_read: max(x.vfs_read, y.vfs_read),
        tcp_send_bytes: max(x.tcp_send_bytes, y.tcp_send_bytes),
        tcp_recv_bytes: max(x.tcp_recv_bytes, y.tcp_recv_bytes),
        l1_dcache_loads: max(x.l1_dcache_loads, y.l1_dcache_loads),
        l1_dcache_load_misses: max(x.l1_dcache_load_misses, y.l1_dcache_load_misses),
        l1_icache_load_misses: max(x.l1_icache_load_misses, y.l1_icache_load_misses),
        llc_load_misses: max(x.llc_load_misses, y.llc_load_misses),
        llc_loads: max(x.llc_loads, y.llc_loads),
        cycles: max(x.cycles, y.cycles),
        instructions: max(x.instructions, y.instructions),
        memory: max(x.memory, y.memory),
    })
}

impl From<measure::ApplicationProfile> for scheduler::Profile {
    fn from(profile: measure::ApplicationProfile) -> Self {
        Self {
            cache_misses: profile.cache_misses as u64,
            cache_references: profile.cache_references as u64,
            vfs_write: profile.vfs_write as u64,
            vfs_read: profile.vfs_read as u64,
            tcp_send_bytes: profile.tcp_send_bytes as u64,
            tcp_recv_bytes: profile.tcp_recv_bytes as u64,
            l1_dcache_loads: profile.l1_dcache_loads as u64,
            l1_dcache_load_misses: profile.l1_dcache_load_misses as u64,
            l1_icache_load_misses: profile.l1_icache_load_misses as u64,
            llc_load_misses: profile.llc_load_misses as u64,
            llc_loads: profile.llc_loads as u64,
            cycles: profile.cycles as u64,
            instructions: profile.instructions as u64,
            memory: profile.memory as u64,
        }
    }
}
