use self::profile::BpfProfile;

use log::debug;
use log::trace;
use log::warn;
use std::error::Error;
use std::io::Write;
use std::process::Stdio;
use std::sync::mpsc::Receiver;
use tempfile::NamedTempFile;

use tokio::process::Command;

pub mod profile;

const BPF_SRC: &str = include_str!("./bpftrace.bp");

pub struct Bpf {
    conf: NamedTempFile,
    receiver: Receiver<bool>,
}

impl Bpf {
    pub fn new(pids: &[u32], receiver: Receiver<bool>) -> Result<Bpf, Box<dyn Error>> {
        let pids: Vec<_> = pids.iter().map(|x| format!("pid == {}", x)).collect();
        let pids = pids.join(" || ");
        let bpf_src = String::from(BPF_SRC).replace("${PID}", &pids[..]);
        trace!("Bpf compiled: \"{}\"", bpf_src);
        let mut file = NamedTempFile::new()?;
        writeln!(file, "{}", bpf_src)?;
        Ok(Bpf { conf: file, receiver })
    }

    pub async fn lop(self) -> Result<Vec<BpfProfile>, Box<dyn Error>> {
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
