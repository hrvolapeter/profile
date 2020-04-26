use clap::{App, Arg};
use disk::run;

fn main() -> disk::BoxResult<()> {
    let matches = App::new("disk")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .required(true)
                .multiple(true)
                .takes_value(true),
        )
        .arg(Arg::with_name("size").short("s").long("size").required(true).takes_value(true))
        .get_matches();

    let paths: Vec<_> = matches.values_of("path").expect("required arg not provided").collect();

    let size = matches.value_of("size").expect("size provided").parse::<u32>()?;

    run(paths, size)?;
    Ok(())
}
