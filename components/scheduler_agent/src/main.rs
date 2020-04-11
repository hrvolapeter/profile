#![deny(warnings)]

mod task;
mod scheduler {
    tonic::include_proto!("scheduler");
}

use machine_id::MachineId;
use scheduler::scheduler_client::SchedulerClient;
use std::cmp::max;
use tonic::transport::Channel;
use tonic::codec::Streaming;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SchedulerClient::connect("http://[::1]:50051").await?;

    let tasks = subscribe_tasks(&mut client).await?;
    let tasks = task::process_tasks(tasks);
    let registration = register(&mut client);
    futures::try_join!(registration, tasks)?;
    Ok(())
}

async fn register(client: &mut SchedulerClient<Channel>) -> Result<(), Box<dyn Error>> {
    let request = tonic::Request::new(scheduler::RegistrationRequest {
        machine_id: MachineId::get().to_string(),
    });

    let response = client.register_server(request).await?.into_inner();
    if response.should_benchmark {
        let profiles = benchmark::run().await?;
        let profile = get_maximum(profiles);
        submit_benchmark(client, profile).await?;
    }
    Ok(())
}

async fn subscribe_tasks(client: &mut SchedulerClient<Channel>) -> Result<Streaming<scheduler::SubscribeTasksReply>, Box<dyn std::error::Error>> {
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

async fn submit_benchmark(
    client: &mut SchedulerClient<Channel>,
    profile: measure::ApplicationProfile,
) -> Result<(), Box<dyn std::error::Error>> {
    let profile = Some(scheduler::Profile {
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
    });
    let request = tonic::Request::new(scheduler::BenchmarkSubmitRequest {
        machine_id: MachineId::get().to_string(),
        profile,
    });

    client.submit_benchmark(request).await?;

    Ok(())
}
