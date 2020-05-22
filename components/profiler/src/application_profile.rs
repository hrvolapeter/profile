use crate::bpf::profile::BpfProfile;
use crate::perf::profile::PerfProfile;
use crate::pmap::PmapProfile;
use crate::BoxResult;
use csv::Writer;
use itertools::izip;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct ApplicationProfile {
    pub cache_misses: u64,
    pub cache_references: u64,
    pub vfs_write: u64,
    pub vfs_read: u64,
    pub tcp_send_bytes: u64,
    pub tcp_recv_bytes: u64,
    //
    pub l1_dcache_loads: u64,
    pub l1_dcache_load_misses: u64,
    pub l1_icache_load_misses: u64,
    pub llc_load_misses: u64,
    pub llc_loads: u64,
    pub cycles: u64,
    pub instructions: u64,
    pub memory: u64,
}

impl ApplicationProfile {
    pub fn new(bpf: Vec<BpfProfile>, perf: Vec<PerfProfile>, pmap: Vec<PmapProfile>) -> Vec<Self> {
        izip!(bpf, perf, pmap)
            .map(|(x, y, z)| ApplicationProfile {
                memory: z.memory,
                cache_misses: x.cache_misses,
                cache_references: x.cache_references,
                vfs_write: x.vfs_write,
                vfs_read: x.vfs_read,
                tcp_send_bytes: x.tcp_send_bytes,
                tcp_recv_bytes: x.tcp_recv_bytes,
                l1_dcache_loads: y.l1_dcache_loads,
                l1_dcache_load_misses: y.l1_dcache_load_misses,
                l1_icache_load_misses: y.l1_icache_load_misses,
                llc_load_misses: y.llc_load_misses,
                llc_loads: y.llc_loads,
                cycles: y.cycles,
                instructions: y.instructions,
            })
            .collect()
    }

    pub fn out(v: Vec<ApplicationProfile>) -> BoxResult<String> {
        let mut wtr = Writer::from_writer(vec![]);
        for i in v {
            wtr.serialize(i)?;
        }

        Ok(String::from_utf8(wtr.into_inner()?).unwrap())
    }
}
