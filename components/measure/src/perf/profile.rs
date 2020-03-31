use from_hashmap::FromHashmap;
use log::trace;
use std::collections::HashMap;
use std::error::Error;
pub trait FromHashmap<T>: Default {
    fn from_hashmap(hm: HashMap<String, u128>) -> T;
}
#[derive(Debug, Default, FromHashmap)]
pub struct PerfProfile {
    pub l1_dcache_loads: u128,
    pub l1_dcache_load_misses: u128,
    pub l1_icache_load_misses: u128,
    pub llc_load_misses: u128,
    pub llc_loads: u128,
    pub cycles: u128,
    pub instructions: u128,
}

type Record = (String, u128, Option<String>, String);

impl PerfProfile {
    pub fn from_stream(s: String) -> Result<Vec<Self>, Box<dyn Error>> {
        trace!("Perf stdout: {}", s);
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(s.as_bytes());
        let res: Vec<Record> = rdr.deserialize().filter_map(Result::ok).collect();

        Ok(build_measurements(transpose(res)))
    }
}

fn build_measurements(mut m: HashMap<String, Vec<u128>>) -> Vec<PerfProfile> {
    let keys: Vec<String> = m.keys().cloned().collect();
    let first_vec = m.keys().next().expect("perf should have returned some results");
    let mut res = vec![];
    for _ in 0..m[first_vec].len() {
        let mut measurement = HashMap::new();
        for k in &keys {
            measurement
                .entry(k.to_lowercase().replace("-", "_"))
                .or_insert_with(|| m.get_mut(&k[..]).unwrap().pop().unwrap());
        }
        res.push(PerfProfile::from_hashmap(measurement));
    }
    res
}

fn transpose(records: Vec<Record>) -> HashMap<String, Vec<u128>> {
    let mut res = HashMap::new();
    for r in records {
        let counter = res.entry(r.3).or_insert_with(|| vec![]);
        counter.push(r.1)
    }
    let l = res.values().next().expect("Perf should have some results").len();
    for (key, val) in &res {
        assert_eq!(l, val.len(), "'{}' different length", key);
    }
    res
}
