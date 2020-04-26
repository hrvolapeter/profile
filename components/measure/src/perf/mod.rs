use self::profile::PerfProfile;
use crate::BoxResult;
use log::debug;
use log::error;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::mpsc::Receiver;
pub mod profile;

const PERF_ARGS: &[&str] = &[
    "stat",
    "-e L1-dcache-loads,L1-dcache-load-misses,L1-icache-load-misses,LLC-loads,LLC-load-misses,cycles,instructions",
];

pub struct Perf {
    pids: String,
    receiver: Receiver<bool>,
}

impl Perf {
    pub fn new(pids: &[u64], receiver: Receiver<bool>) -> Self {
        let pids: Vec<_> = pids.iter().map(|x| x.to_string()).collect();
        let pids = pids.join(",");
        Self { pids, receiver }
    }

    pub async fn run(mut self) -> BoxResult<Vec<PerfProfile>> {
        let cmd = Command::new("/usr/bin/perf")
            .args(PERF_ARGS)
            .arg(format!("-p {}", &self.pids[..]))
            .arg("-I 10000")
            .arg("-x,")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.receiver.recv().await;
        stop_process(&cmd)?;

        let out = cmd.wait_with_output().await?;
        if !out.status.success() {
            error!("Perf exited with error: {:?}", out.status);
        }
        debug!("Perfs exited");
        PerfProfile::from_stream(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

fn stop_process(process: &Child) -> Result<(), impl std::error::Error> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let pid = Pid::from_raw(process.id() as i32);
    kill(pid, Signal::SIGTERM)
}
