// #![deny(warnings)]

mod flow;
mod web;

use std::error::Error;
use web::graph::Graph;
use tokio::sync::watch;


#[tokio::main(core_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;
    let mut graph = flow::Graph::default();
    graph.add_server(flow::Server::new("Server 1".to_string(), Default::default()));
    graph.add_server(flow::Server::new("Server 2".to_string(), Default::default()));
    graph.add_server(flow::Server::new("Server 3".to_string(), Default::default()));

    graph.add_task(flow::Task::new("Task 1".to_string(), Default::default()));
    graph.add_task(flow::Task::new("Task 2".to_string(), Default::default()));
    graph.add_task(flow::Task::new("Task 3".to_string(), Default::default()));
    let graph = Graph::from_flow(graph.run());
    let (_tx, rx) =  watch::channel(graph);
    let server = web::serve(rx);
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
