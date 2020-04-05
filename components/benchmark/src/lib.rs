#![deny(warnings)]

use std::error::Error;
use measure::application_profile::ApplicationProfile;

pub async fn run() -> Result<Vec<ApplicationProfile>, Box<dyn Error>> {
    let f = Box::new(move || {
        workload().unwrap();
    });
    measure::run(None, Some(f)).await
}

fn workload() -> Result<(), Box<dyn Error>> {
    cpu::run()?;
    memory::run(2)?;
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