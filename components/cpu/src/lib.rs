use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::error::Error;
pub type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn run() -> BoxResult<()> {
    let s = include_str!("./numbers.txt");
    let res: Vec<_> = s
        .lines()
        .par_bridge()
        .map(|i| {
            let num: u128 = i.trim().parse().unwrap();
            get_factors_functional(num)
        })
        .collect();

    eprintln!("{}", res.len());
    Ok(())
}

fn get_factors_functional(n: u128) -> Vec<u128> {
    (1..=n).filter(|&x| n % x == 0).collect::<Vec<_>>()
}
