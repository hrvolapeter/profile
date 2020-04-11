use std::error::Error;
use tonic::codec::Streaming;
use crate::scheduler;
use tokio::stream::StreamExt;


pub async fn process_tasks(mut tasks: Streaming<scheduler::SubscribeTasksReply>) -> Result<(), Box<dyn Error>> {
    while let Some(x) = tasks.next().await {
        dbg!(x?);
    }
    Ok(())
}