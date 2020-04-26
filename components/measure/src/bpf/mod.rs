use self::profile::BpfProfile;

use log::debug;
use log::trace;
use log::warn;
use std::io::Write;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::sync::mpsc::Receiver;

use crate::BoxResult;
use tokio::process::Command;

pub mod profile;

const BPF_SRC: &str = include_str!("./bpftrace.bp");

pub struct Bpf {
    conf: NamedTempFile,
    receiver: Receiver<bool>,
}

impl Bpf {
    pub fn new(pids: &[u64], receiver: Receiver<bool>) -> BoxResult<Bpf> {
        let pids: Vec<_> = pids.iter().map(|x| format!("pid == {}", x)).collect();
        let pids = pids.join(" || ");
        let bpf_src = String::from(BPF_SRC).replace("${PID}", &pids[..]);
        trace!("Bpf compiled: \"{}\"", bpf_src);
        let mut file = NamedTempFile::new()?;
        writeln!(file, "{}", bpf_src)?;
        Ok(Bpf { conf: file, receiver })
    }

    pub async fn lop(mut self) -> BoxResult<Vec<BpfProfile>> {
        let mut res = vec![];
        while self.receiver.try_recv().is_err() {
            let cmd = Command::new("/usr/bin/bpftrace")
                .arg(self.conf.path())
                .arg("-fjson")
                .stdout(Stdio::piped())
                .spawn()?;

            let out = cmd.wait_with_output().await?;
            let s = String::from_utf8_lossy(&out.stdout).to_string();
            if let Ok(profile) = BpfProfile::from_stream(&s) {
                res.push(profile);
            } else {
                warn!("No bpf parsed from: \"{}\"", s);
            }
        }
        debug!("Bpfs exited");
        Ok(res)
    }
}
