#![warn(clippy::all)]

use clap::{App, Arg};
use std::fs::File;
use std::io::{stdin, stdout, Write};

use stalch::Error::*;
use stalch::*;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("SOURCE").help("Source code to run").required_unless("interactive"))
        .arg(
            Arg::with_name("interactive")
                .short("i")
                .long("interactive")
                .help("Starts interactive shell"),
        )
        .get_matches();
    let mut state = State::new();
    let mut stdouter = InOuter::new(stdout(), stdin());

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
                break;
            }
            match run_with_state(s.as_bytes(), &mut state, &mut stdouter) {
                Ok(()) => (),
                Err(e) => handle_error(e),
            }
            println!(" >{:?}", state.show_stack());
        }
    } else {
        let src = matches.value_of("SOURCE").unwrap();

        let file = File::open(src).unwrap();
        match run_with_state(file, &mut state, &mut stdouter) {
            Ok(()) => (),
            Err(e) => handle_error(e),
        }
    }
}

fn handle_error(e: Error) {
    match e {
        IoError(e) => panic!("Unexpected error:\n{:?}", e),
        CharsError(e) => panic!("Unexpected error:\n{:?}", e),
        EmptyStack => eprintln!("Error, empty stack"),
        OutOfBounds => eprintln!("Error, out of bounds"),
        InvalidAssignArg => eprintln!("Error, can only assign value to a variable name"),
        InvalidApplyArg => eprintln!("Error, can only execute blocks"),
        InvalidSplitArg => eprintln!("Error, split takes a number and a block or string"),
        InvalidGetArg => eprintln!("Error, get takes a number and a block or string"),
        InvalidMoveArg => eprintln!("Error, move takes a number and one other value"),
        InvalidGrabArg => eprintln!("Error, can only take number as grab argument"),
        InvalidIncludeArg => eprintln!("Error, include can only take a string"),
        NoBlockStarted => eprintln!("Error, cannot end a block when none has been started"),
    }
}
