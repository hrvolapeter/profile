use clap::{App, Arg};
use memory::run;

fn main() -> memory::BoxResult<()> {
    let matches = App::new("memory")
        .arg(Arg::with_name("size").short("s").long("size").required(true).takes_value(true))
        .get_matches();

    let size = matches.value_of("size").expect("size provided").parse::<u32>()?;
    run(size)?;
    Ok(())
}
