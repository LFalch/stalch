use std::fs::File;
use std::io::{Write, Read, BufRead, BufReader};

mod chars;
mod value;
mod cmd;
mod state;
mod err;

use crate::chars::*;
use crate::value::Value;
use crate::value::Value::*;
use crate::cmd::Command;
use crate::cmd::Command::*;
pub use crate::state::State;
pub use crate::err::{Result, Error};

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
        Value(s) => state.push(s),
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
        Pack => {
            let mut to_push = Vec::new();

            for val in state.drain_stack() {
                match val {
                    Block(n, cmds) => {
                        to_push.push(BeginBlock);
                        for _ in 0..n {
                            for cmd in &cmds {
                                to_push.push(cmd.clone());
                            }
                        }
                        to_push.push(EndBlock);
                    }
                    v => to_push.push(Value(v)),
                }
            }

            state.push(Block(1, to_push));
        }
        Size => {
            let size = state.stack().len() as i128;
            state.push(Integer(size))
        }
        Length => {
            let to_push = match *state.peek()? {
                Block(n, ref b) => Integer((n as usize * b.len()) as i128),
                Str(ref s) => Integer(s.chars().count() as i128),
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
        Define => {
            match (state.pop_pure()?, state.pop()?) {
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
                s @ Str(_) => state.push(s),
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
            let i = match state.pop()? {
                Integer(n) => n,
                _ => return Err(Error::InvalidSplitArg)
            } as usize;

            match state.pop()?.flatten() {
                Block(n, mut b) => {
                    debug_assert_eq!(n, 1, "value has been flattened");

                    let i = b.len().checked_sub(i).ok_or(Error::OutOfBounds)?;

                    let right = b.split_off(i);
                    state.push(Block(1, b));
                    state.push(Block(1, right));
                }
                Str(mut s) => {
                    let len = s.chars().count();
                    let byte_len = s.len();

                    let i = len.checked_sub(i).ok_or(Error::OutOfBounds)?;

                    let real_index = s.char_indices().chain(Some((byte_len, '\x00'))).nth(i).ok_or(Error::OutOfBounds)?.0;
                    let right = s.split_off(real_index);
                    state.push(Str(s));
                    state.push(Str(right));
                }
                _ => return Err(Error::InvalidSplitArg)
            }
        }
        Get => {
            let i = match state.pop()? {
                Integer(n) => n,
                _ => return Err(Error::InvalidGetArg)
            } as usize;

            match state.pop()?.flatten() {
                Block(n, mut b) => {
                    debug_assert_eq!(n, 1, "value has been flattened");

                    let i = b.len().checked_sub(i+1).ok_or(Error::OutOfBounds)?;

                    let cmd = b.remove(i);
                    state.push(Block(1, b));
                    state.push(Block(1, vec![cmd]));
                }
                Str(mut s) => {
                    let len = s.chars().count();
                    let byte_len = s.len();

                    let i = len.checked_sub(i).ok_or(Error::OutOfBounds)?;

                    let real_index = s.char_indices().chain(Some((byte_len, '\x00'))).nth(i).ok_or(Error::OutOfBounds)?.0;
                    let right = s.split_off(real_index);

                    let c = s.pop().ok_or(Error::OutOfBounds)?.to_string();

                    s += &right;

                    state.push(Str(s));
                    state.push(Str(c));
                }
                _ => return Err(Error::InvalidGetArg)
            }
        }
        DupGet => {
            let i = match state.pop()? {
                Integer(n) => n,
                _ => return Err(Error::InvalidGetArg)
            } as usize;

            let val = match state.peek()? {
                Block(n, b) => {
                    let len = *n as usize * b.len();
                    let i = len.checked_sub(i+1).ok_or(Error::OutOfBounds)?;

                    let real_index = i % b.len();

                    let cmd = b[real_index].clone();

                    Block(1, vec![cmd])
                }
                Str(s) => {
                    let c = s.chars().rev().nth(i).ok_or(Error::OutOfBounds)?;

                    Str(c.to_string())
                }
                _ => return Err(Error::InvalidGetArg)
            };

            state.push(val);
        }
        Move => match state.pop()? {
            Integer(n) => {
                let elem = state.peek()?.clone();

                state.insert(n as usize, elem)?;
                state.pop()?;
            },
            _ => return Err(Error::InvalidMoveArg)
        }
        Grab => match state.pop()? {
            Integer(n) => {
                let n_elem = state.take_nth(n as usize)?;

                state.push(n_elem);
            },
            _ => return Err(Error::InvalidGrabArg)
        }
        DupGrab => match state.pop()? {
            Integer(n) => {
                let n_elem = state.nth(n as usize)?.clone();

                state.push(n_elem);
            },
            _ => return Err(Error::InvalidGrabArg)
        }
        Drop => {
            state.pop()?;
        }
        Type => {
            let t = match state.pop()? {
                Float(_) => "float",
                Integer(_) => "int",
                Bool(_) => "bool",
                Str(_) => "str",
                Variable(_) => "var",
                Block(_, _) => "block",
                Null => "null",
            }.into();
            state.push(t);
        }
        ToFloat => state.last_mut()?.make_float(),
        ToInt => state.last_mut()?.make_int(),
        ToBool => state.last_mut()?.make_bool(),
        Eq => binop(state, |a, b| a == b)?,
        Neq => binop(state, |a, b| a != b)?,
        GreaterThan => binop(state, |a, b| a > b)?,
        GreaterEquals => binop(state, |a, b| a >= b)?,
        LessThan => binop(state, |a, b| a < b)?,
        LessEquals => binop(state, |a, b| a <= b)?,
        Write => write!(io.o, "{}", state.pop()?)?,
        Print => writeln!(io.o, "{}", state.pop()?)?,
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
