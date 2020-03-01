use serde::{Deserialize, Serialize};
use std::error::Error;


#[derive(Serialize, Deserialize, Debug)]
pub struct ResCount {
    #[serde(alias = "cache-misses:10000")]
    cache_misses: u128,
    #[serde(alias = "cache-references:10000")]
    cache_references: u128,
    #[serde(alias = "instructions:10000")]
    instructions: u128,
    #[serde(alias = "cycles:10000")]
    cycles: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResSum {
    vfs_write: u128,
    vfs_read: u128,
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
    pub vfs_write: Option<u128>,
    pub vfs_read: Option<u128>,
}

#[derive(Debug, Default)]
pub struct BpfProfileBuilder {
    pub cycles: Option<u128>,
    pub instructions: Option<u128>,
    pub cache_misses: Option<u128>,
    pub cache_references: Option<u128>,
    pub vfs_write: Option<u128>,
    pub vfs_read: Option<u128>,
}

impl BpfProfileBuilder {
    fn build(self) -> BpfProfile {
        BpfProfile {
            cycles: self.cycles.unwrap(),
            instructions: self.instructions.unwrap(),
            cache_misses: self.cache_misses.unwrap(),
            cache_references: self.cache_references.unwrap(),
            vfs_read: self.vfs_read,
            vfs_write: self.vfs_write,
        }
    }
}

impl BpfProfile {
    pub fn from_stream(s: String) -> Result<Self, Box<dyn Error>> {
        let res: Vec<_> = s
            .lines()
            .map(|x| serde_json::from_str(x))
            .filter_map(Result::ok)
            .map(|x| match x {
                Entry::Map_(BpfOut::ResCount(x)) => BpfOut::ResCount(normalize_counts(x, 10_000)),
                Entry::Map_(x) => x,
            })
            .collect();

        assert!(
            res.len() < 2 && !res.is_empty(),
            "should have at least counts at most counts, sums once"
        );
        let mut out = BpfProfileBuilder {
            ..Default::default()
        };
        for i in res {
            match i {
                BpfOut::ResCount(x) => {
                    out.cycles = Some(x.cycles);
                    out.instructions = Some(x.instructions);
                    out.cache_misses = Some(x.cache_misses);
                    out.cache_references = Some(x.cache_references);
                }
                BpfOut::ResSum(x) => {
                    out.vfs_write = Some(x.vfs_write);
                    out.vfs_read = Some(x.vfs_read);
                }
            }
        }
        Ok(out.build())
    }
}

fn normalize_counts(x: ResCount, factor: u128) -> ResCount {
    ResCount {
        cache_misses: x.cache_misses * factor,
        cache_references: x.cache_references * factor,
        instructions: x.instructions * factor,
        cycles: x.cycles * factor,
    }
}
