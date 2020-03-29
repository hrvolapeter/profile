#![feature(test)]

use clap::{App, Arg};
use rand::Rng;
use std::error::Error;

use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    run_test()
}

fn run_test() -> Result<(), Box<dyn Error>> {
    let matches = App::new("memory")
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("size")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let size = matches
        .value_of("size")
        .expect("size provided")
        .parse::<u32>()?;

    let handles: Vec<_> = (0..num_cpus::get())
        .map(|_| thread::spawn(move || benchmark(size).unwrap()))
        .collect();

    for h in handles {
        h.join().unwrap();
    }
    Ok(())
}

fn benchmark(size: u32) -> Result<u32, Box<dyn Error>> {
    let elements = size * 100_000_000;
    let mut vector = Vec::with_capacity(elements as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..elements {
        vector.push(rng.gen::<u32>());
    }
    eprintln!("Starting computing");
    let mut res = 0;
    for _ in 0..1_000_000_000 {
        let i = rng.gen_range(0, elements) as usize;
        let num = unsafe { vector.get_unchecked(i) };
        res = bencher::black_box(*num);
    }
    eprintln!("Done computing");
    Ok(res)
}
