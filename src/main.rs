extern crate clap;
extern crate esogolf;

use std::fs::File;
use clap::{Arg, App};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
                          .version(env!("CARGO_PKG_VERSION"))
                          .author(env!("CARGO_PKG_AUTHORS"))
                          .about(env!("CARGO_PKG_DESCRIPTION"))
                          .arg(Arg::with_name("SOURCE")
                               .help("Source code to run")
                               .required(true))
                          .arg(Arg::with_name("v")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity"))
                          .get_matches();
    let src = matches.value_of("SOURCE").unwrap();

    let file = File::open(src).unwrap();
    esogolf::run_program(file);
}
