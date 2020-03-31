#![feature(test)]

use clap::{App, Arg};
use memory::run;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("memory")
        .arg(Arg::with_name("size").short("s").long("size").required(true).takes_value(true))
        .get_matches();

    let size = matches.value_of("size").expect("size provided").parse::<u32>()?;
    run(size)?;
    Ok(())
}
