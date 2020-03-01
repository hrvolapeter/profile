#![feature(async_closure)]
#![feature(try_trait)]

use crate::application_profile::ApplicationProfile;
use crate::bpf::Bpf;
use crate::perf::Perf;
use std::env;
use std::error::Error;
use std::sync::mpsc::channel;
use tokio::process::Child;
use clap::{App, Arg};
use tokio::process::Command;
use tokio::spawn;
use futures::future::join_all;
use log::debug;
use std::sync::mpsc::Sender;
use fern::colors::ColoredLevelConfig;
use tokio::task::JoinHandle;

mod application_profile;
mod bpf;
mod perf;
mod util;
mod out;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;
    let matches = App::new("measure")
        .arg(Arg::with_name("pid").short('p').long("pid").multiple(true).takes_value(true))
        .arg(Arg::with_name("app").multiple(true).last(true))
        .get_matches();

    let mut main: Option<Child> = None;
    let pids: Vec<_> = if let Some(x) = matches.values_of("pid") {
        let pids = x.map(|x| x.parse::<u32>().expect("Pid must be number") ).collect();
        debug!("Pids provided, registering {:?}", pids);
        pids
    } else {
        let args = matches.values_of("app").unwrap().collect();
        main = Some(execute_main(args)?);
        vec![main.as_ref().unwrap().id()]
    };

    let mut bpfs = vec![];
    let mut bpfs_senders = vec![];
    let mut perfs = vec![];
    let mut perfs_senders = vec![];
    for pid in pids  {
        let (sender, receiver) = channel();
        bpfs_senders.push(sender);
        let bpf = Bpf::new(pid, receiver)?.lop();
        bpfs.push(spawn(async {bpf.await.unwrap()}));
        
        let (sender, receiver) = channel();
        perfs_senders.push(sender);
        let perf = Perf::new(pid, receiver).run();
        perfs.push(spawn(async {perf.await.unwrap()}));
    }

    if let Some(main) = main {
        main.await?;
        exit_tracers(&bpfs_senders, &perfs_senders);
    } 
    
    let (sender, receiver) = channel();
    ctrlc::set_handler(move || {
        debug!("received Ctrl+C!");
        exit_tracers(&bpfs_senders, &perfs_senders);
        sender.send(true);
    })?;
    receiver.recv()?;
    print_profile(bpfs, perfs).await;
    Ok(())
}
use crate::bpf::profile::BpfProfile;
use crate::perf::profile::PerfProfile;
async fn print_profile(bpfs: Vec<JoinHandle<Vec<BpfProfile>>>, perfs:Vec<JoinHandle<Vec<PerfProfile>>>) {
    let bpfs = join_all(bpfs).await.into_iter().filter_map(Result::ok).flatten().collect();
    let perfs = join_all(perfs).await.into_iter().filter_map(Result::ok).flatten().collect();
    let ap = ApplicationProfile::new(bpfs, perfs);
    println!("{:?}", ap);

}

fn exit_tracers(bpfs: &Vec<Sender<bool>>, perfs: &Vec<Sender<bool>> ) {
    for sender in bpfs {
        sender.send(true).unwrap();
    }
    for sender in perfs {
        sender.send(true).unwrap();
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new().debug(fern::colors::Color::Green);
    
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
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn execute_main(args: Vec<&str>) -> Result<Child, impl Error> {
    Command::new(args[0]).args(&args[1..]).spawn()
}
