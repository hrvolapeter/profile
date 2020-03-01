
use crate::bpf::profile::BpfProfile;
use crate::perf;
use crate::perf::profile::PerfProfile;


#[derive(Debug)]
pub struct ApplicationProfile {
    bpf: BpfProfile,
    perf: perf::profile::Measurement,
}

impl ApplicationProfile {
    pub fn new(bpf: Vec<BpfProfile>, perf: PerfProfile) -> Vec<Self> {
        bpf.into_iter()
            .zip(perf.measurements.into_iter())
            .map(|(x, y)| ApplicationProfile { bpf: x, perf: y })
            .collect()
    }
}
