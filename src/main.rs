use crate::application_profile::ApplicationProfile;
use crate::bpf::Bpf;
use crate::perf::Perf;
use futures::{
    future::FutureExt, // for `.fuse()`
    pin_mut,
    select,
};
use std::env;
use std::error::Error;
use std::sync::mpsc::channel;
use tokio::process::Child;
use tokio::process::Command;

mod application_profile;
mod bpf;
mod perf;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let main = execute_main()?;

    let (sender, receiver) = channel();
    let bpf = Bpf::new(main.id(), receiver)?.lop().fuse();
    let perf = Perf::new(main.id()).run()?;
    let main = main.fuse();

    pin_mut!(main, bpf);
    let mut bpf_profile = None;

    loop {
        select!(
            _ = main => sender.send(true)?,
            x = bpf => bpf_profile = Some(x?),
            complete => break,
        );
    }

    let perf_profile = perf.stop().await?;
    let application_profile = ApplicationProfile::new(bpf_profile.unwrap(), perf_profile);
    println!("{:#?}", application_profile);
    Ok(())
}

fn execute_main() -> Result<Child, impl Error> {
    let mut args = env::args();
    args.next().unwrap();
    let program = args.next().unwrap();

    Command::new(program).args(args).spawn()
}
