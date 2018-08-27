use std::collections::HashMap;

use value::Value;
use cmd::Command;
use err::*;

pub struct State {
    stack: Vec<Value>,
    pub block_nesting: u8,
    vars: HashMap<String, Value>,
    pub temp: Vec<Command>,
}

impl State {
    pub fn new() -> Self {
        State {
            block_nesting: 0,
            stack: Vec::new(),
            vars: HashMap::new(),
            temp: Vec::new(),
        }
    }
    #[inline(always)]
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }
    #[inline(always)]
    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::EmptyStack)
    }
    #[inline(always)]
    pub fn peek(&self) -> Result<&Value> {
        self.stack.last().ok_or(Error::EmptyStack)
    }
    #[inline]
    fn index(&self, n: usize) -> Result<usize> {
        self.stack.len().checked_sub(n+1).ok_or(Error::OutOfBounds)
    }
    #[inline]
    pub fn insert(&mut self, n: usize, val: Value) -> Result<()> {
        self.index(n)
            .map(|i| self.stack.insert(i, val))
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
