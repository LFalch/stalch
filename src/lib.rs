#![feature(io)]

use std::fs::File;
use std::io::Read;

struct State {
    stack: Vec<Value>
}

impl State {
    fn new() -> Self{
        State{stack: Vec::new()}
    }
}

pub fn run_program(src_file: File) {
    let mut buf = String::new();
    let mut state = State::new();
    let mut is_making_string = false;

    for c in src_file.chars() {
        match c {
            Ok(c) => {
                if c.is_whitespace() && !is_making_string {
                    if !buf.is_empty() {
                        run_command(&mut state, &buf);
                        buf.clear();
                    }
                } else {
                    if c == '"' {
                        is_making_string = !is_making_string;
                    }
                    buf.push(c);
                }
            },
            Err(e) => panic!("{:?}", e)
        }
    }
}

mod value;
use value::Value;
use value::Value::*;

fn run_command(state: &mut State, cmd: &str) {
    if cmd.starts_with('"') {
        state.stack.push(Str(cmd[1..cmd.len()-1].to_owned()));
    } else if let Ok(n) = cmd.parse::<f64>() {
        state.stack.push(Num(n));
    } else if let "*0" = cmd {
        state.stack.push(Value::Null);
    } else {
        match cmd {
            "dup" => {
                let dup = state.stack.last().cloned().unwrap();
                state.stack.push(dup);
            },
            "prnt" => {
                match state.stack.pop().unwrap() {
                    Str(s) => println!("{}", s),
                    Num(n) => println!("{}", n),
                    Null => println!("NULL")
                }
            }
            _ => panic!("Unknown cmd `{}'", cmd)
        }
    }
}
