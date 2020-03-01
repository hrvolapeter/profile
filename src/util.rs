use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::error::Error;
use tokio::process::Child;

pub fn stop_process(process: &mut Child) -> Result<(), impl Error> {
    let pid = Pid::from_raw(process.id() as i32);
    kill(pid, Signal::SIGINT)
}
