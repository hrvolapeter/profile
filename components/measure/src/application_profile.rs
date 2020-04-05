use crate::bpf::profile::BpfProfile;
use crate::perf::profile::PerfProfile;
use crate::pmap::PmapProfile;
use csv::Writer;
use itertools::izip;
use serde::Serialize;
use std::error::Error;

#[derive(Debug, Serialize)]
pub struct ApplicationProfile {
    cache_misses: u128,
    cache_references: u128,
    vfs_write: u128,
    vfs_read: u128,
    tcp_send_bytes: u128,
    tcp_recv_bytes: u128,
    //
    l1_dcache_loads: u128,
    l1_dcache_load_misses: u128,
    l1_icache_load_misses: u128,
    llc_load_misses: u128,
    llc_loads: u128,
    cycles: u128,
    instructions: u128,
    memory: u128,
}

impl ApplicationProfile {
    pub fn new(bpf: Vec<BpfProfile>, perf: Vec<PerfProfile>, pmap: Vec<PmapProfile>) -> Vec<Self> {
        assert_eq!(bpf.len(), perf.len(), "Should have equal number of profiels");
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

    pub fn out(v: Vec<ApplicationProfile>) -> Result<String, Box<dyn Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        for i in v {
            wtr.serialize(i)?;
        }

        Ok(String::from_utf8(wtr.into_inner()?).unwrap())
    }
}
