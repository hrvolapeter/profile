#![feature(async_closure)]
#![feature(try_trait)]

use crate::application_profile::ApplicationProfile;
use crate::bpf::profile::BpfProfile;
use crate::bpf::Bpf;
use crate::perf::profile::PerfProfile;
use crate::perf::Perf;
use crate::pmap::Pmap;
use crate::pmap::PmapProfile;
use clap::{App, Arg};
use fern::colors::ColoredLevelConfig;
use futures::future::join_all;
use log::debug;
use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use tokio::process::Child;
use tokio::process::Command;
use tokio::spawn;
use tokio::task::JoinHandle;

mod application_profile;
mod bpf;
mod perf;
mod pmap;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;
    let matches = App::new("measure")
        .arg(
            Arg::with_name("pid")
                .short('p')
                .long("pid")
                .multiple(true)
                .takes_value(true),
        )
        .arg(Arg::with_name("app").multiple(true).last(true))
        .get_matches();

    let mut main: Option<Child> = None;
    let pids: Vec<_> = if let Some(x) = matches.values_of("pid") {
        let pids = x
            .map(|x| x.parse::<u32>().expect("Pid must be number"))
            .collect();
        debug!("Pids provided, registering {:?}", pids);
        pids
    } else {
        let args = matches
            .values_of("app")
            .expect("Required argument missing")
            .collect();
        main = Some(execute_main(args)?);
        vec![main.as_ref().unwrap().id()]
    };

    let mut bpfs = vec![];
    let mut bpfs_senders = vec![];
    let (sender, receiver) = channel();
    bpfs_senders.push(sender);
    let bpf = Bpf::new(&pids[..], receiver)?.lop();
    bpfs.push(spawn(async { bpf.await.unwrap() }));

    let mut pmaps = vec![];
    let mut pmaps_senders = vec![];
    let (sender, receiver) = channel();
    pmaps_senders.push(sender);
    let pmap = Pmap::new(&pids[..], receiver)?.lop();
    pmaps.push(spawn(async { pmap.await.unwrap() }));

    let mut perfs = vec![];
    let mut perfs_senders = vec![];
    let (sender, receiver) = channel();
    perfs_senders.push(sender);
    let perf = Perf::new(&pids[..], receiver).run();
    perfs.push(spawn(async { perf.await.unwrap() }));

    let (sender, receiver) = channel();

    if let Some(main) = main {
        debug!("Waiting for main to finish");
        main.await?;
        debug!("Main finished, exiting perf and bpf");
        exit_tracers(&bpfs_senders, &perfs_senders, &pmaps_senders);
        sender.send(true)?;
    }

    ctrlc::set_handler(move || {
        debug!("received Ctrl+C!");
        exit_tracers(&bpfs_senders, &perfs_senders, &pmaps_senders);
        sender.send(true).expect("Send close message");
    })?;
    receiver.recv()?;
    print_profile(bpfs, perfs, pmaps).await;
    Ok(())
}

async fn print_profile(
    bpfs: Vec<JoinHandle<Vec<BpfProfile>>>,
    perfs: Vec<JoinHandle<Vec<PerfProfile>>>,
    pmaps: Vec<JoinHandle<Vec<PmapProfile>>>,
) {
    let bpfs = join_all(bpfs)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect();
    let pmaps = join_all(pmaps)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect();
    let perfs = join_all(perfs)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect();
    let ap = ApplicationProfile::new(bpfs, perfs, pmaps);
    println!("{}", ApplicationProfile::out(ap).unwrap());
}

fn exit_tracers(bpfs: &Vec<Sender<bool>>, perfs: &Vec<Sender<bool>>, pmap: &Vec<Sender<bool>>) {
    for sender in bpfs {
        sender.send(true).unwrap();
    }
    for sender in perfs {
        sender.send(true).unwrap();
    }
    for sender in pmap {
        sender.send(true).unwrap();
    }
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

fn execute_main(args: Vec<&str>) -> Result<Child, impl Error> {
    Command::new(args[0]).args(&args[1..]).spawn()
}
