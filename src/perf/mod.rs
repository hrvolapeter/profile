use self::profile::PerfProfile;
use crate::util::stop_process;
use std::error::Error;
use std::process::Stdio;
use tokio::process::Command;
use std::sync::mpsc::Receiver;
use log::error;
use log::debug;

pub mod profile;

const PERF_ARGS: &[&str] = &[
    "stat",
    "-e L1-dcache-loads,L1-dcache-load-misses,L1-icache-load-misses,LLC-loads,LLC-load-misses,cycles,instructions"
];

pub struct Perf {
    pid: u32,
    receiver: Receiver<bool>,

}

impl Perf {
    pub fn new(pid: u32, receiver: Receiver<bool>) -> Self {
        Self { pid, receiver}
    }

    pub async fn run(self) -> Result<Vec<PerfProfile>, Box<dyn Error>> {
        let cmd = Command::new("/usr/bin/perf")
            .args(PERF_ARGS)
            .arg(format!("-p {}", self.pid))
            .arg("-I 5000")
            .arg("-x,")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.receiver.recv()?;
        stop_process(&cmd)?;

        let out = cmd.wait_with_output().await?;
        if !out.status.success() {
            error!("Perf exited with error {:?}", out);
        }
        debug!("Perfs exited");
        PerfProfile::from_stream(String::from_utf8_lossy(&out.stderr).to_string())
    }
}
