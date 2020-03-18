use clap::{App, Arg};
use rand::Rng;
use std::error::Error;
use std::thread;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use ring::digest::{Context, Digest, SHA256};

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

 
    let handles: Vec<_> = paths.clone().map(|x| {
        let x = x.to_string();
        let size = size.clone();
        thread::spawn(move || benchmark_write(x, size).unwrap())
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    let handles: Vec<_> = paths.map(|x| {
        let x = x.to_string();
        thread::spawn(move || benchmark_read(x).unwrap())
    }).collect();

    for h in handles {
        let res = h.join().unwrap();
        eprintln!("{:?}", res);
    }
    Ok(())
}

fn benchmark_write(p: String, size: u32) -> Result<(), Box<dyn Error>> {
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

fn benchmark_read(p: String) -> Result<Digest, Box<dyn Error>> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 4096*4];
    let file = OpenOptions::new().read(true).open(p)?;
    let mut rdr = BufReader::new(file);
    loop {
        let count = rdr.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok(context.finish())
}
