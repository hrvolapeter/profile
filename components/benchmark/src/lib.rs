#![deny(warnings)]

use profiler::ApplicationProfile;
use std::error::Error;
pub type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub async fn run() -> BoxResult<Vec<ApplicationProfile>> {
    let f = Box::new(move || {
        workload().unwrap();
    });
    profiler::run(None, Some(f), None).await
}

fn workload() -> BoxResult<()> {
    cpu::run()?;
    memory::run(1)?;
    let paths = vec![
        "/tmp/bench.1",
        "/tmp/bench.2",
        "/tmp/bench.3",
        "/tmp/bench.4",
        "/tmp/bench.5",
        "/tmp/bench.6",
    ];
    disk::run(paths, 1)?;
    network::run();
    Ok(())
}
