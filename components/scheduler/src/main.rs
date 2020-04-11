#![deny(warnings)]
#![allow(dead_code)]
#![feature(async_closure)]
#![feature(try_trait)]

mod scheduler;
mod webui;
mod rpc;

use futures_util::future::FutureExt;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

#[tokio::main(core_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;
    let scheduler = Arc::new(Mutex::new(scheduler::Scheduler::new()));

    let http_server = webui::serve(scheduler.clone());

    let addr = "[::]:50051".parse()?;
    let rpc_server = Server::builder()
        .add_service(rpc::SchedulerServer::new(rpc::SchedulerService::new(scheduler.clone())))
        .serve(addr)
        .map(|_| ());

    futures::join!(http_server, rpc_server);
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
        .level(log::LevelFilter::Warn)
        .level_for("scheduler", log::LevelFilter::Trace)
        .apply()?;
    Ok(())
}

mod import {
    #[allow(warnings)]
    pub(crate) use {
        log::debug, std::collections::HashMap, std::path::Path, std::sync::Arc,
        tokio::sync::Mutex, serde::Serialize, serde::Deserialize, log::error,
    };
    pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;
}
