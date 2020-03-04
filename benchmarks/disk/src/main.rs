use clap::{App, Arg};
use rand::Rng;
use std::error::Error;
use std::thread;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("disk")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .required(true)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("size")
                .required(true)
                .takes_value(true)
        )
        .get_matches();

    let paths = matches
        .values_of("path")
        .expect("required arg not provided");

    let size = matches
        .value_of("size")
        .expect("size provided").parse::<u32>()?;

 
    let handles: Vec<_> = paths.map(|x| {
        let x = x.to_string();
        let size = size.clone();
        thread::spawn(move || benchmark(x, size).unwrap())
    }).collect();

    for h in handles {
        h.join().unwrap();
    }
    Ok(())
}

fn benchmark(p: String, size: u32) -> Result<(), Box<dyn Error>> {
    let mut vector = Vec::new();
    
    let mut rng = rand::thread_rng();
    for _ in 0..(size * 10_000_000) {
        vector.push(rng.gen::<char>());
    }
    let string: String = vector.iter().collect();

    let file = OpenOptions::new().create(true).write(true).truncate(true).open(p)?;
    let mut wrt = BufWriter::new(file);
    for _ in 0..100 {
        wrt.write_all(string.as_bytes())?;
    }
    wrt.flush()?;
    Ok(())
}
