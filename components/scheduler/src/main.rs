// #![deny(warnings)]

mod flow;
mod web;

use pharos::Pharos;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use web::graph::Graph;

#[tokio::main(core_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;
    let graph = flow::Graph::default();
    let paths = graph.run();
    Graph::from_flow(paths);
    let pharos = Arc::new(Mutex::new(Pharos::default()));
    let server = web::serve(pharos.clone());
    futures::join!(server);
    Ok(())
}

fn setup_logger() -> Result<(), fern::InitError> {
    use fern::colors::ColoredLevelConfig;
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
