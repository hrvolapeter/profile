use std::fs;
use std::env;
use std::error::Error;
use rayon::prelude::*;
use rayon::iter::ParallelBridge;
use clap::{App, Arg};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("cpu")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let s = fs::read_to_string(matches.value_of("path").unwrap())?;
    let res: Vec<_> = s.lines().par_bridge().map(|i| {
        let num: u128 = i.trim().parse().unwrap();
        get_factors_functional(num)
    }).collect();

    eprintln!("{}", res.len());
    Ok(())
}

fn get_factors_functional(n: u128) -> Vec<u128> {
    (1..=n).filter(|&x| n % x == 0).collect::<Vec<_>>()
}
