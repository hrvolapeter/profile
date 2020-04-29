#![feature(thread_id_value)]
#![deny(warnings)]
#![feature(async_closure)]

mod application_profile;
mod bpf;
mod perf;
mod pmap;

pub use crate::application_profile::ApplicationProfile;
use crate::bpf::Bpf;
use crate::perf::Perf;
use crate::pmap::Pmap;
use futures::executor::block_on;
use std::error::Error;
use log::debug;
use std::thread;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub async fn run(
    pids: Option<Vec<u64>>,
    main: Option<Box<dyn FnOnce() -> () + Send>>,
    cancel_channel: Option<Receiver<()>>,
) -> BoxResult<Vec<ApplicationProfile>> {
    // Prepare plumbing
    let (ctrlc_sender, mut ctrlc_receiver) = channel(10);
    let (bpf_sender, bpf_receiver) = channel(10);
    let (pmap_sender, pmap_receiver) = channel(10);
    let (perf_sender, perf_receiver) = channel(10);
    let mut _handle = None;

    let pids: Vec<u64> = if let Some(x) = pids {
        debug!("Pids provided, registering {:?}", x);
        x.to_vec()
    } else {
        let main = main.expect("Required argument missing");
        let mut bpf_sender = bpf_sender.clone();
        let mut perf_sender = perf_sender.clone();
        let mut pmap_sender = pmap_sender.clone();
        let mut ctrlc_sender = ctrlc_sender.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        let h = thread::Builder::new().name("main_bench".to_string()).spawn(move || {
            tx.send(std::process::id()).unwrap();
            main();
            block_on(exit_tracers(&mut [
                &mut bpf_sender,
                &mut perf_sender,
                &mut pmap_sender,
                &mut ctrlc_sender,
            ]));
        })?;
        _handle = Some(h);
        let pid = rx.recv()?;
        let res = vec![pid as u64];
        debug!("Started main with: {:?}", res);
        res
    };

    let bpf = Bpf::new(&pids[..], bpf_receiver)?.lop();
    let pmap = Pmap::new(&pids[..], pmap_receiver)?.lop();
    let perf = Perf::new(&pids[..], perf_receiver).run();
    let senders: Vec<_> =
        [bpf_sender, perf_sender, pmap_sender, ctrlc_sender].to_vec();

    if let Some(mut cancel_channel) = cancel_channel {
        tokio::spawn(async move {
            cancel_channel.recv().await;
            for mut sender in senders {
                sender.send(true).await.unwrap();
            }
        });
    }

    let (bpf, perf, pmap, _) = futures::join!(bpf, perf, pmap, ctrlc_receiver.recv());
    let ap = ApplicationProfile::new(bpf?, perf?, pmap?);
    Ok(ap)
}

async fn exit_tracers(senders: &mut [&mut Sender<bool>]) {
    for sender in senders {
        sender.send(true).await.unwrap();
    }
}