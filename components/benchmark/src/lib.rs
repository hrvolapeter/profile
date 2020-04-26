#![deny(warnings)]

use measure::ApplicationProfile;
use std::error::Error;
pub type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub async fn run() -> BoxResult<Vec<ApplicationProfile>> {
    let f = Box::new(move || {
        workload().unwrap();
    });
    measure::run(None, Some(f), None).await
}

fn workload() -> BoxResult<()> {
    cpu::run()?;
    memory::run(1)?;
    let paths = vec![
        "/mnt/nvme/bench.1",
        "/mnt/nvme/bench.2",
        "/mnt/nvme/bench.3",
        "/mnt/nvme/bench.4",
        "/mnt/nvme/bench.5",
        "/mnt/nvme/bench.6",
    ];
    disk::run(paths, 2)?;
    network::run();
    Ok(())
}
