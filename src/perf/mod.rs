use self::profile::PerfProfile;
use crate::util::stop_process;
use std::error::Error;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;

pub mod profile;

const PERF_ARGS: &[&str] = &[
    "stat",
    "-e L1-dcache-loads,L1-dcache-load-misses,L1-icache-load-misses,LLC-loads,LLC-load-misses,cycles,instructions"
];

pub struct Perf {
    pid: u32,
}

impl Perf {
    pub fn new(pid: u32) -> Self {
        Self { pid }
    }

    pub fn run(self) -> Result<PerfRunning, Box<dyn Error>> {
        let cmd = Command::new("/usr/bin/perf")
            .args(PERF_ARGS)
            .arg(format!("-p {}", self.pid))
            .arg("-I 1000")
            .arg("-x,")
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(PerfRunning { exec: cmd })
    }
}

pub struct PerfRunning {
    exec: Child,
}

impl PerfRunning {
    pub async fn stop(mut self) -> Result<PerfProfile, Box<dyn Error>> {
        stop_process(&mut self.exec)?;
        let out = self.exec.wait_with_output().await?;
        PerfProfile::from_stream(String::from_utf8_lossy(&out.stderr).to_string())
    }
}
