use rand::Rng;
use ring::digest::{Context, Digest, SHA256};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::thread;
pub type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn run(paths: Vec<&str>, size: u32) -> BoxResult<()> {
    let handles: Vec<_> = paths
        .iter()
        .clone()
        .map(|x| {
            let x = (*x).to_string();
            let size = size.clone();
            thread::spawn(move || benchmark_write(x, size).unwrap())
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let handles: Vec<_> = paths
        .iter()
        .map(|x| {
            let x = (*x).to_string();
            thread::spawn(move || benchmark_read(x).unwrap())
        })
        .collect();

    for h in handles {
        let res = h.join().unwrap();
        eprintln!("{:?}", res);
    }
    Ok(())
}

fn benchmark_write(p: String, size: u32) -> BoxResult<()> {
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

fn benchmark_read(p: String) -> BoxResult<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 4096 * 4];
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
