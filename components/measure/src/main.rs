#![deny(warnings)]

use clap::{App, Arg};
use log::debug;
use measure::ApplicationProfile;
use std::error::Error;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("measure")
        .arg(Arg::with_name("pid").short('p').long("pid").multiple(true).takes_value(true))
        .arg(Arg::with_name("app").multiple(true).last(true))
        .get_matches();

    let pids: Option<Vec<_>> = matches
        .values_of("pid")
        .map(|pids| pids.map(|x| x.parse::<u64>().expect("Pid must be number")).collect());

    let args: Option<Vec<String>> =
        matches.values_of("app").map(|args| args.map(|x| x.to_string()).collect());
    debug!("args : {:?}", args);
    if let Some(args) = args {
        let f = Box::new(move || {
            Command::new(args[0].clone()).args(&args[1..]).output().unwrap();
        });
        let ap = measure::run(None, Some(f)).await?;
        println!("{}", ApplicationProfile::out(ap).unwrap());
        return Ok(());
    }

    let ap = measure::run(pids, None::<Box<dyn FnOnce() -> () + Send>>).await?;
    println!("{}", ApplicationProfile::out(ap).unwrap());
    Ok(())
}
