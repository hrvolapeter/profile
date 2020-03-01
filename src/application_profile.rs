
use crate::bpf::profile::BpfProfile;
use crate::perf;
use crate::perf::profile::PerfProfile;


#[derive(Debug)]
pub struct ApplicationProfile {
    bpf: BpfProfile,
    perf: PerfProfile,
}

impl ApplicationProfile {
    pub fn new(bpf: Vec<BpfProfile>, perf: Vec<PerfProfile>) -> Vec<Self> {
        bpf.into_iter()
            .zip(perf.into_iter())
            .map(|(x, y)| ApplicationProfile { bpf: x, perf: y })
            .collect()
    }
}
