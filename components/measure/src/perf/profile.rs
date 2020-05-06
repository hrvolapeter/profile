use crate::BoxResult;
use log::trace;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PerfProfile {
    pub l1_dcache_loads: u64,
    pub l1_dcache_load_misses: u64,
    pub l1_icache_load_misses: u64,
    pub llc_load_misses: u64,
    pub llc_loads: u64,
    pub cycles: u64,
    pub instructions: u64,
}

type Record = (String, String, Option<String>, String);

impl PerfProfile {
    pub fn from_stream(s: String) -> BoxResult<Vec<Self>> {
        trace!("Perf stdout: {}", s);
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).flexible(true).from_reader(s.as_bytes());
        let res: Vec<Record> = rdr.deserialize().filter_map(Result::ok).collect();
        trace!("Res {:?}", &rdr.deserialize::<Record>().collect::<Vec<_>>());
        let res = group_by_key(res);
        trace!("Grouped {:?}", res);
        Ok(res
            .into_iter()
            .map(|x| PerfProfile {
                l1_dcache_loads: *x.get("L1-dcache-load").unwrap_or(&0),
                l1_dcache_load_misses: *x.get("L1-dcache-load-misses").unwrap_or(&0),
                l1_icache_load_misses: *x.get("L1-icache-load-misses").unwrap_or(&0),
                llc_load_misses: *x.get("LLC-load-misses").unwrap_or(&0),
                llc_loads: *x.get("LLC-load").unwrap_or(&0),
                cycles: *x.get("cycles").unwrap_or(&0),
                instructions: *x.get("instructions").unwrap_or(&0),
            })
            .collect())
    }
}

fn group_by_key(rows: Vec<Record>) -> Vec<HashMap<String, u64>> {
    let mut res = vec![];
    let mut profile = HashMap::new();
    let mut time = rows[0].0.clone();
    for row in rows {
        if time != row.0 {
            res.push(profile);
            profile = HashMap::new();
            time = row.0;
        }
        profile.insert(row.3, row.1.parse::<u64>().unwrap_or(0));
    }
    res.push(profile);
    res
}
