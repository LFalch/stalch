#![feature(io)]

use std::fs::File;
use std::io::Read;

struct State {
    stack: Vec<Value>,
    block_nesting: u8,
    temp: Vec<Command>
}

impl State {
    fn new() -> Self{
        State {
            block_nesting: 0,
            stack: Vec::new(),
            temp: Vec::new()
        }
    }
}

pub fn run_program(src_file: File) {
    let mut buf = String::new();
    let mut state = State::new();
    let mut ignoring_whitespace = false;

    for c in src_file.chars() {
        match c {
            Ok(c) => {
                if c.is_whitespace() && !ignoring_whitespace {
                    if !buf.is_empty() {
                        run_command(&mut state, Command::from_str(&buf));
                        buf.clear();
                    }
                } else {
                    if c == '"' {
                        ignoring_whitespace = !ignoring_whitespace;
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

mod cmd;
use cmd::Command;
use cmd::Command::*;

fn binop<T: Into<Value>, F: FnOnce(Value, Value) -> T>(s: &mut State, f: F) {
    let b = s.stack.pop().unwrap();
    let a = s.stack.pop().unwrap();

    s.stack.push(f(a, b).into())
}

use std::ops;
use std::mem::replace;

fn run_command(state: &mut State, cmd: Command) {
    if cfg!(feature = "debug") {
        println!("{f}  {indent}{:?}: {:?}", cmd, state.stack,
            f = if cmd == BeginBlock {"\n"}else{""},
            indent = "    ".repeat((state.block_nesting as usize)
            .saturating_sub(if cmd == EndBlock {1}else{0})),
        );
    }

    match cmd {
        EndBlock => match state.block_nesting {
            0 => panic!("Can't end block when none is begun"),
            1 => {
                state.block_nesting = 0;

                let t = replace(&mut state.temp, Vec::new());
                state.stack.push(Block(1, t));
            }
            _ => {
                state.block_nesting -= 1;
                state.temp.push(EndBlock);
            }
        }
        BeginBlock => {
            state.block_nesting += 1;
            if state.block_nesting > 1 {
                state.temp.push(BeginBlock);
            }
        }
        ref cmd if state.block_nesting > 0 => state.temp.push(cmd.clone()),
        Value(s) => if s.starts_with('"') {
            state.stack.push(Str(s[1..s.len()-1].to_owned()));
        } else if let Ok(n) = s.parse::<f64>() {
            state.stack.push(Num(n));
        } else {
            panic!("Unknown cmd `{}'", s)
        }
        EmptyBlock => state.stack.push(Block(1, Vec::new())),
        True => state.stack.push(Num(1.)),
        False => state.stack.push(Num(0.)),
        NullVal => state.stack.push(Value::Null),
        Dup => {
            let dup = state.stack.last().cloned().unwrap();
            state.stack.push(dup);
        }
        Not => {
            let a = state.stack.pop().unwrap();
            state.stack.push(!a);
        }
        If => {
            let when_false = state.stack.pop().unwrap();
            let when_true = state.stack.pop().unwrap();
            let condition = state.stack.pop().unwrap();

            state.stack.push(if condition.as_bool() {
                when_true
            } else {
                when_false
            });
        }
        ApplyFunction => {
            match state.stack.pop() {
                Some(Block(n, b)) => {
                    for _ in 0..n {
                        for cmd in &b {
                            run_command(state, cmd.clone());
                        }
                    }
                }
                _ => panic!("No block on stack")
            }
        }
        Read => {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            line = line.trim_right().to_owned();
            state.stack.push(Str(line));
        }
        Swap => {
            let a = state.stack.pop().unwrap();
            let b = state.stack.pop().unwrap();
            state.stack.push(a);
            state.stack.push(b);
        }
        Drop => {
            state.stack.pop().unwrap();
        }
        CastNum => state.stack.last_mut().unwrap().make_num(),
        Eq => binop(state, |a, b| a == b),
        Neq => binop(state, |a, b| a != b),
        Write => print!("{}", state.stack.pop().unwrap()),
        Print => println!("{}", state.stack.pop().unwrap()),
        Or => binop(state, ops::BitOr::bitor),
        And => binop(state, ops::BitAnd::bitand),
        Add => binop(state, ops::Add::add),
        Sub => binop(state, ops::Sub::sub),
        Mul => binop(state, ops::Mul::mul),
        Div => binop(state, ops::Div::div),
        Rem => binop(state, ops::Rem::rem)
    }
}
