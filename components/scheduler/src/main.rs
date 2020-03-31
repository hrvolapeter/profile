#![deny(warnings)]
#![allow(dead_code)]

mod flow;
mod web;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main(core_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;
    let scheduler = Arc::new(Mutex::new(flow::Scheduler::new()));

    let server = web::serve(scheduler.clone());
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

mod import {
    #[allow(warnings)]
    pub(crate) use {
        std::collections::HashMap, std::error::Error, std::sync::Arc, tokio::sync::Mutex,
    };
}
