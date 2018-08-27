use std::fs::File;
use std::io::{Write, Read, BufRead, BufReader};

mod chars;
mod value;
mod cmd;
mod state;
mod err;

use chars::*;
use value::Value;
use value::Value::*;
use cmd::Command;
use cmd::Command::*;
pub use state::State;
pub use err::{Result, Error};

pub struct InOuter<W: Write, R: Read> {
    o: W,
    i: BufReader<R>,
}

impl<W: Write, R: Read> InOuter<W, R> {
    pub fn new(o: W, i: R) -> Self {
        InOuter {
            o,
            i: BufReader::new(i),
        }
    }
    pub fn extract(self) -> (W, R) {
        let InOuter{i, o} = self;
        (o, i.into_inner())
    }
}

pub fn run_with_state<R, R2, W>(src: R, state: &mut State, io: &mut InOuter<W, R2>) -> Result<()>
where R: Read, R2: Read, W: Write {
    let mut buf = String::new();
    let mut ignoring_whitespace = false;

    for c in src.chars_iterator() {
        match c {
            Ok(c) => {
                if c.is_whitespace() && !ignoring_whitespace {
                    if !buf.is_empty() {
                        run_command(state, Command::from_str(&buf), io)?;
                        buf.clear();
                    }
                } else {
                    if c == '"' {
                        ignoring_whitespace = !ignoring_whitespace;
                    }
                    buf.push(c);
                }
            },
            Err(e) => return Err(Error::CharsError(e))
        }
    }

    Ok(())
}

fn binop<T: Into<Value>, F: FnOnce(Value, Value) -> T>(s: &mut State, f: F) -> Result<()> {
    let b = s.pop()?;
    let a = s.pop()?;

    s.push(f(a, b).into());
    Ok(())
}

use std::ops;
use std::mem::replace;

fn run_command<W: Write, R: Read>(state: &mut State, cmd: Command, io: &mut InOuter<W, R>) -> Result<()> {
    if cfg!(feature = "debug") {
        println!("{f}  {indent}{:?}: {:?}", cmd, state.stack(),
            f = if cmd == BeginBlock {"\n"}else{""},
            indent = "    ".repeat((state.block_nesting as usize)
            .saturating_sub(if cmd == EndBlock {1}else{0})),
        );
    }

    match cmd {
        EndBlock => match state.block_nesting {
            0 => return Err(Error::NoBlockStarted),
            1 => {
                state.block_nesting = 0;

                let t = replace(&mut state.temp, Vec::new());
                state.push(Block(1, t));
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
            state.push(Str(s[1..s.len()-1].to_owned()));
        } else if let Ok(n) = s.parse::<f64>() {
            state.push(Num(n));
        } else {
            let to_push;
            if let Some(v) = state.get_var(&s) {
                to_push = v.clone();
            } else {
                to_push = Variable(s);
            }
            state.push(to_push);
        }
        EmptyBlock => state.push(Block(1, Vec::new())),
        Include => {
            match state.pop()? {
                Str(s) => {
                    let file = File::open(s)?;
                    run_with_state(file, state, io)?;
                }
                _ => return Err(Error::InvalidIncludeArg)
            }
        }
        True => state.push(Num(1.)),
        False => state.push(Num(0.)),
        NullVal => state.push(Value::Null),
        Size => {
            let size = state.stack().len() as f64;
            state.push(Num(size))
        }
        Length => {
            let to_push = match *state.peek()? {
                Block(n, ref b) => Num((n as usize * b.len()) as f64),
                Str(ref s) => Num(s.len() as f64),
                _ => Null
            };
            state.push(to_push);
        }
        Dup => {
            let dup = state.peek()?.clone();
            state.push(dup);
        }
        Not => {
            let a = state.pop()?;
            state.push(!a);
        }
        If => {
            let when_false = state.pop()?;
            let when_true = state.pop()?;
            let condition = state.pop()?;

            state.push(if condition.as_bool() {
                when_true
            } else {
                when_false
            });
        }
        Assign => {
            match (state.pop()?, state.pop()?) {
                (Variable(_), Variable(_)) => return Err(Error::InvalidAssignArg),
                (Variable(h), v) | (v, Variable(h)) => state.add_var(h, v),
                _ => return Err(Error::InvalidAssignArg),
            }
        }
        ApplyFunction => {
            match state.pop()? {
                Block(n, b) => {
                    for _ in 0..n {
                        for cmd in &b {
                            run_command(state, cmd.clone(), io)?;
                        }
                    }
                }
                _ => return Err(Error::InvalidApplyArg)
            }
        }
        Read => {
            let mut line = String::new();
            io.i.read_line(&mut line)?;
            line = line.trim_right().to_owned();
            state.push(Str(line));
        }
        Swap => {
            let a = state.pop()?;
            let b = state.pop()?;
            state.push(a);
            state.push(b);
        }
        Split => {
            match state.pop()? {
                Block(n, b) => {
                    let bl = n as f64 * b;
                    bl
                }
                Str(s) => {

                }
                _ => return Err(Error::InvalidSplitArg)
            }
        }
        Get => {

        }
        DupGet => {

        }
        Move => {

        }
        Grab => match state.pop()? {
            Num(n) => {
                let n_elem = state.take_nth(n as usize)?;

                state.push(n_elem);
            },
            _ => return Err(Error::InvalidGrabArg)
        }
        DupGrab => match state.pop()? {
            Num(n) => {
                let n_elem = state.nth(n as usize)?.clone();

                state.push(n_elem);
            },
            _ => return Err(Error::InvalidGrabArg)
        }
        Drop => {
            state.pop()?;
        }
        CastNum => state.last_mut()?.make_num(),
        Eq => binop(state, |a, b| a == b)?,
        Neq => binop(state, |a, b| a != b)?,
        Write => write!(io.o, "{}", state.pop()?)?,
        Print => write!(io.o, "{}\n", state.pop()?)?,
        Or => binop(state, ops::BitOr::bitor)?,
        And => binop(state, ops::BitAnd::bitand)?,
        Add => binop(state, ops::Add::add)?,
        Sub => binop(state, ops::Sub::sub)?,
        Mul => binop(state, ops::Mul::mul)?,
        Div => binop(state, ops::Div::div)?,
        Rem => binop(state, ops::Rem::rem)?
    }

    Ok(())
}
