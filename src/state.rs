use std::collections::HashMap;
use std::fmt::{self, Debug};

use crate::cmd::Command;
use crate::err::*;
use crate::value::Value;

#[derive(Debug, Default)]
pub struct State {
    stack: Vec<Value>,
    pub block_nesting: u8,
    vars: HashMap<String, Value>,
    pub temp: Vec<Command>,
}

impl State {
    pub fn new() -> Self {
        State::default()
    }
    pub fn drain_stack(&mut self) -> impl Iterator<Item = Value> + '_ {
        self.stack.drain(..)
    }
    #[inline(always)]
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
    pub fn show_stack(&self) -> ShowState<'_> {
        ShowState(self)
    }
    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }
    #[inline(always)]
    pub fn pop_pure(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::EmptyStack)
    }
    pub fn pop(&mut self) -> Result<Value> {
        self.pop_pure().map(|v| {
            if let Value::Variable(v) = v {
                if let Some(v) = self.get_var(&v) {
                    v.clone()
                } else {
                    Value::Variable(v)
                }
            } else {
                v
            }
        })
    }
    #[inline(always)]
    pub fn peek_pure(&self) -> Result<&Value> {
        self.stack.last().ok_or(Error::EmptyStack)
    }
    #[inline(always)]
    pub fn peek(&self) -> Result<&Value> {
        self.peek_pure().map(|v| {
            if let Value::Variable(v) = v {
                if let Some(v) = self.get_var(&v) {
                    v
                } else {
                    // TODO HACK FIX this is bad code
                    self.peek_pure().unwrap()
                }
            } else {
                v
            }
        })
    }
    #[inline]
    fn index(&self, n: usize) -> Result<usize> {
        self.stack.len().checked_sub(n + 1).ok_or(Error::OutOfBounds)
    }
    #[inline]
    pub fn insert(&mut self, n: usize, val: Value) -> Result<()> {
        self.index(n).map(|i| self.stack.insert(i, val))
    }
    pub fn nth(&self, n: usize) -> Result<&Value> {
        Ok(&self.stack[self.index(n)?])
    }
    #[inline(always)]
    pub fn last_mut(&mut self) -> Result<&mut Value> {
        self.stack.last_mut().ok_or(Error::EmptyStack)
    }
    pub fn take_nth(&mut self, n: usize) -> Result<Value> {
        let index = self.index(n)?;
        Ok(self.stack.remove(index))
    }
    #[inline(always)]
    pub fn get_var(&self, var: &str) -> Option<&Value> {
        self.vars.get(var)
    }
    #[inline(always)]
    pub fn add_var(&mut self, var: String, val: Value) {
        self.vars.insert(var, val);
    }
}

pub struct ShowState<'a>(&'a State);

impl Debug for ShowState<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_list();
        for v in &self.0.stack {
            match v {
                Value::Variable(s) => {
                    if let Some(v) = self.0.get_var(s) {
                        dbg.entry(v);
                    } else {
                        dbg.entry(&format_args!("{}", s));
                    }
                }
                _ => {
                    dbg.entry(v);
                }
            }
        }
        dbg.finish()
    }
}
