use crate::scheduler;
use std::error::Error;
use tokio::stream::StreamExt;
use tonic::codec::Streaming;

pub async fn process_tasks(
    mut tasks: Streaming<scheduler::SubscribeTasksReply>,
) -> Result<(), Box<dyn Error>> {
    while let Some(x) = tasks.next().await {
        dbg!(x?);
    }
    Ok(())
}
