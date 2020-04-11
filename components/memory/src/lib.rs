#![feature(test)]

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::error::Error;
use std::thread;

/// Arguments:
///
/// * `size`: Amount of data wirtten to memory in TODO: (what unit?)
pub fn run(size: u32) -> Result<(), Box<dyn Error>> {
    let handles: Vec<_> =
        (0..num_cpus::get()).map(|_| thread::spawn(move || benchmark(size).unwrap())).collect();

    for h in handles {
        h.join().unwrap();
    }
    Ok(())
}

fn benchmark(size: u32) -> Result<u32, Box<dyn Error>> {
    let mut rng = SmallRng::from_entropy();

    let mut small_vector = Vec::with_capacity(1000);
    for _ in 0..1000 {
        small_vector.push(rng.gen::<u32>());
    }

    let elements = size * 100_000;
    let mut big_vector = Vec::with_capacity(elements as usize);
    for _ in 0..elements {
        big_vector.append(&mut small_vector.clone());
    }

    eprintln!("Starting computing");
    let mut res = 0;
    let mut indices = vec![];
    for _ in 0..1000 {
        indices.push(rng.gen_range(0, elements) as usize);
    }
    for _ in 0..1_000_000 {
        for i in &indices {
            let num = unsafe { big_vector.get_unchecked(*i) };
            res = bencher::black_box(*num);
        }
    }
    eprintln!("Done computing");
    Ok(res)
}
