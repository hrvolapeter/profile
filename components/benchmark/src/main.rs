use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    cpu::run()?;
    memory::run(10)?;
    let paths = vec![
        "/tmp/bench.1",
        "/tmp/bench.2",
        "/tmp/bench.3",
        "/tmp/bench.4",
        "/tmp/bench.5",
        "/tmp/bench.6",
    ];
    disk::run(paths, 10)?;
    network::run();
    Ok(())
}
