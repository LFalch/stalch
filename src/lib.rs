#![feature(io)]

use std::fs::File;
use std::io::Read;

struct State {
    stack: Vec<Value>,
    wip_block: Option<Vec<String>>
}

impl State {
    fn new() -> Self{
        State {
            stack: Vec::new(),
            wip_block: None
        }
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

fn binop<F: FnOnce(Value, Value) -> Value>(s: &mut State, f: F) {
    let a = s.stack.pop().unwrap();
    let b = s.stack.pop().unwrap();

    s.stack.push(f(a, b))
}

use std::ops::*;

fn run_command(state: &mut State, cmd: &str) {
    if let ref mut b @ Some(_) = state.wip_block {
        if cmd == "}" || cmd == "]" || cmd == "end" {
            state.stack.push(Block(b.take().unwrap()));
        } else {
            b.as_mut().unwrap().push(cmd.to_owned());
        }
    } else if cmd.starts_with('"') {
        state.stack.push(Str(cmd[1..cmd.len()-1].to_owned()));
    } else if let Ok(n) = cmd.parse::<f64>() {
        state.stack.push(Num(n));
    } else {
        match &*cmd.to_lowercase() {
            "{" | "[" | "do" => state.wip_block = Some(Vec::new()),
            "T" | "true" => state.stack.push(Num(1.)),
            "t" | "f" | "false" => state.stack.push(Num(0.)),
            "¤" | "null" | "nil" => state.stack.push(Value::Null),
            ";" | "dup" => {
                let dup = state.stack.last().cloned().unwrap();
                state.stack.push(dup);
            }
            "!" | "not" => {
                let a = state.stack.pop().unwrap();
                state.stack.push(!a);
            }
            "if" | "?" => {
                let condition = state.stack.pop().unwrap();
                let when_false = state.stack.pop().unwrap();
                let when_true = state.stack.pop().unwrap();

                state.stack.push(if condition.as_bool() {
                    when_true
                } else {
                    when_false
                });
            }
            "()" => {
                match state.stack.pop() {
                    Some(Block(b)) => {
                        for cmd in b {
                            run_command(state, &cmd);
                        }
                    }
                    _ => panic!("No block on stack")
                }
            }
            "<" => state.stack.push(Str("user input".to_owned())),
            ">" | "wrte" => print!("{}", state.stack.pop().unwrap()),
            "_" | "prnt" => println!("{}", state.stack.pop().unwrap()),
            "+" | "add" => binop(state, Add::add),
            "-" | "sub" => binop(state, Sub::sub),
            "*" | "mul" => binop(state, Mul::mul),
            "/" | "div" => binop(state, Div::div),
            "%" | "rem" => binop(state, Rem::rem),
            _ => panic!("Unknown cmd `{}'", cmd)
        }
    }
}
