extern crate clap;
extern crate stalch;

use std::fs::File;
use clap::{Arg, App};

use std::io::{stdin, stdout, Write};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
                          .version(env!("CARGO_PKG_VERSION"))
                          .author(env!("CARGO_PKG_AUTHORS"))
                          .about(env!("CARGO_PKG_DESCRIPTION"))
                          .arg(Arg::with_name("SOURCE")
                               .help("Source code to run")
                               .required_unless("interactive"))
                          .arg(Arg::with_name("interactive")
                               .short("i")
                               .long("interactive")
                               .help("Starts interactive shell"))
                          .get_matches();
    let mut state = stalch::State::new();

    if matches.is_present("interactive") {
        println!("Stalch Interactive Shell");
        println!("Type $exit to exit");
        loop {
            print!("$> ");
            stdout().flush().unwrap();

            let mut s = String::new();
            stdin().read_line(&mut s).unwrap();
            if s.trim_right() == "$exit" {
                println!();
                break
            }
            stalch::run_with_state(s.as_bytes(), &mut state);
            println!(" >{:?}", state.stack);
        }
    } else {
        let src = matches.value_of("SOURCE").unwrap();

        let file = File::open(src).unwrap();
        stalch::run_with_state(file, &mut state);
    }

}
