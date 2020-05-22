#![deny(warnings)]

use clap::{App, Arg};
use fern::colors::ColoredLevelConfig;
use futures::executor::block_on;
use log::debug;
use profiler::ApplicationProfile;
use std::error::Error;
use std::process::Command;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    setup_logger()?;
    let matches = App::new("profiler")
        .arg(Arg::with_name("pid").short('p').long("pid").multiple(true).takes_value(true))
        .arg(Arg::with_name("app").multiple(true).last(true))
        .get_matches();

    let pids: Option<Vec<_>> = matches
        .values_of("pid")
        .map(|pids| pids.map(|x| x.parse::<u64>().expect("Pid must be number")).collect());

    let args: Option<Vec<String>> =
        matches.values_of("app").map(|args| args.map(|x| x.to_string()).collect());
    debug!("args : {:?}", args);
    let (sender, receiver) = mpsc::channel(10);
    ctrlc::set_handler(move || {
        debug!("received Ctrl+C!");
        block_on(sender.clone().send(())).unwrap();
    })?;

    if let Some(args) = args {
        let f = Box::new(move || {
            Command::new(args[0].clone()).args(&args[1..]).output().unwrap();
        });
        let ap = profiler::run(None, Some(f), Some(receiver)).await?;
        println!("{}", ApplicationProfile::out(ap).unwrap());
        return Ok(());
    }

    let ap = profiler::run(pids, None::<Box<dyn FnOnce() -> () + Send>>, Some(receiver)).await?;
    println!("{}", ApplicationProfile::out(ap).unwrap());
    Ok(())
}

fn setup_logger() -> Result<(), fern::InitError> {
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
