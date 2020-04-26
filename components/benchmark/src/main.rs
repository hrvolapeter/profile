#[tokio::main]
async fn main() -> benchmark::BoxResult<()> {
    benchmark::run().await?;
    Ok(())
}
