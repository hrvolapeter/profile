use crate::BoxResult;
use log::debug;
use log::trace;
use regex::Regex;
use std::process::Stdio;
use std::{thread, time};
use tokio::process::Command;
use tokio::sync::mpsc::Receiver;

pub struct Pmap {
    pids: Vec<String>,
    receiver: Receiver<bool>,
}

impl Pmap {
    pub fn new(pids: &[u64], receiver: Receiver<bool>) -> BoxResult<Pmap> {
        let pids: Vec<_> = pids.iter().map(|x| x.to_string()).collect();
        Ok(Self { pids, receiver })
    }

    pub async fn lop(mut self) -> BoxResult<Vec<PmapProfile>> {
        let mut res = vec![];
        let regex = Regex::new(r"total kB\s+(\d+)").unwrap();
        while self.receiver.try_recv().is_err() {
            thread::sleep(time::Duration::from_secs(10));

            let cmd = Command::new("/usr/bin/pmap")
                .arg("-x")
                .args(&self.pids)
                .stdout(Stdio::piped())
                .spawn()?;

            let out = cmd.wait_with_output().await?;
            let s = String::from_utf8_lossy(&out.stdout).to_string();

            let mut memory = 0;
            for cap in regex.captures_iter(&s[..]) {
                trace!("Pmap result regex {:?}", cap);
                memory += cap
                    .get(1)
                    .map(|x| x.as_str().parse::<u64>().expect("Should be number"))
                    .expect("Have number");
            }

            let pf = PmapProfile { memory };
            debug!("Pmap {:?}", pf);
            res.push(pf);
        }
        debug!("Pmap exited");
        Ok(res)
    }
}

#[derive(Debug)]
pub struct PmapProfile {
    pub memory: u64,
}
