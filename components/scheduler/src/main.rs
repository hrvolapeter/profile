#![deny(warnings)]
#![allow(dead_code)]
#![feature(async_closure)]
#![feature(try_trait)]

mod rpc;
mod scheduler;
mod webui;

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
        .apply()?;
    Ok(())
}

mod import {
    #[allow(warnings)]
    pub(crate) use {
        log::debug, log::error, log::trace, serde::Deserialize, serde::Serialize,
        std::collections::HashMap, std::path::Path, std::sync::Arc, tokio::sync::Mutex, uuid::Uuid, rust_decimal::Decimal, std::str::FromStr,
    };
    pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;
}
