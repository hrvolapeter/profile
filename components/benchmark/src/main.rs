use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    benchmark::run().await?;
    Ok(())
}
