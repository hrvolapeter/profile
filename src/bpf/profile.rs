use serde::{Deserialize, Serialize};
use std::error::Error;
use log::trace;



#[derive(Serialize, Deserialize, Debug)]
pub struct ResCount {
    #[serde(alias = "cache-misses:10000")]
    cache_misses: Option<u128>,
    #[serde(alias = "cache-references:10000")]
    cache_references: u128,
    #[serde(alias = "instructions:10000")]
    instructions: u128,
    #[serde(alias = "cycles:10000")]
    cycles: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResSum {
    vfs_write: Option<u128>,
    vfs_read: Option<u128>,
    tcp_send_bytes: Option<u128>,
    tcp_recv_bytes: Option<u128>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BpfOut {
    #[serde(alias = "@res_count")]
    ResCount(ResCount),
    #[serde(alias = "@res_sum")]
    ResSum(ResSum),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum Entry {
    #[serde(alias = "map")]
    Map_(BpfOut),
}

#[derive(Debug)]
pub struct BpfProfile {
    pub cycles: u128,
    pub instructions: u128,
    pub cache_misses: u128,
    pub cache_references: u128,
    pub vfs_write: u128,
    pub vfs_read: u128,
    pub tcp_send_bytes: u128,
    pub tcp_recv_bytes: u128,
}

#[derive(Debug, Default)]
pub struct BpfProfileBuilder {
    pub cycles: Option<u128>,
    pub instructions: Option<u128>,
    pub cache_misses: Option<u128>,
    pub cache_references: Option<u128>,
    pub vfs_write: Option<u128>,
    pub vfs_read: Option<u128>,
    pub tcp_send_bytes: Option<u128>,
    pub tcp_recv_bytes: Option<u128>,
}

impl BpfProfileBuilder {
    fn build(self) -> BpfProfile {
        BpfProfile {
            cycles: self.cycles.unwrap_or_default(),
            instructions: self.instructions.unwrap_or_default(),
            cache_misses: self.cache_misses.unwrap_or_default(),
            cache_references: self.cache_references.unwrap_or_default(),
            vfs_read: self.vfs_read.unwrap_or_default(),
            vfs_write: self.vfs_write.unwrap_or_default(),
            tcp_send_bytes: self.tcp_send_bytes.unwrap_or_default(),
            tcp_recv_bytes: self.tcp_recv_bytes.unwrap_or_default(),
        }
    }
}

impl BpfProfile {
    pub fn from_stream(s: &String) -> Result<Self, Box<dyn Error>> {
        trace!("Bpf stdout message: {:?}", &s);
        let res: Vec<_> = s
            .lines()
            .map(|x| serde_json::from_str(x))
            .filter_map(Result::ok)
            .map(| Entry::Map_(x) | x)
            .collect();
        assert!(
            res.len() < 3 ,
            "should have at most counts, sums once"
        );
        let mut out = BpfProfileBuilder::default();
        for i in res {
            trace!("Bpf parsed message: \"{:?}\"", i);
            match i {
                BpfOut::ResCount(x) => {
                    out.cycles = Some(x.cycles);
                    out.instructions = Some(x.instructions);
                    out.cache_misses = x.cache_misses;
                    out.cache_references = Some(x.cache_references);
                }
                BpfOut::ResSum(x) => {
                    out.vfs_write = x.vfs_write;
                    out.vfs_read = x.vfs_read;
                    out.tcp_send_bytes = x.tcp_send_bytes;
                    out.tcp_recv_bytes = x.tcp_recv_bytes;
                }
            }
        }
        Ok(out.build().normalize(10_000))
    }

    fn normalize(mut self, factor: u128) -> Self {  
        self.cache_misses = self.cache_misses * factor;
        self.cache_references = self.cache_references * factor;
        self.instructions = self.instructions * factor;
        self.cycles = self.cycles * factor;

        self
    }
    
}