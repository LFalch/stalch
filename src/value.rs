use std::cmp::Ordering;
use std::f64::NAN;
use std::fmt;
use std::ops::*;

use crate::cmd::Command;

#[derive(Clone)]
pub enum Value {
    Float(f64),
    Integer(i64),
    Bool(bool),
    Str(String),
    Variable(String),
    Block(u16, Vec<Command>),
    Null,
}

impl Value {
    pub fn parse(s: &str) -> Self {
        if s.starts_with('"') {
            debug_assert!(s.len() > 1, "string {:?} has invalid format", s);
            Str(s[1..s.len() - 1].to_owned())
        } else if let Ok(n) = s.parse::<i64>() {
            Integer(n)
        } else if let Ok(n) = s.parse::<f64>() {
            Float(n)
        } else if let Ok(b) = s.parse::<bool>() {
            Bool(b)
        } else if s == "ø" || s == "null" {
            Null
        } else {
            Variable(s.to_owned())
        }
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Float(n) => !n.is_nan(),
            Integer(_) => true,
            Str(ref s) => !s.is_empty(),
            Bool(b) => b,
            Block(_, _) => true,
            // TODO Return error
            Variable(_) => false,
            Null => false,
        }
    }
    pub fn make_bool(&mut self) {
        let repl = self.as_bool();
        *self = Bool(repl);
    }
    pub fn make_float(&mut self) {
        let repl = match *self {
            Bool(b) => Float(f64::from(b as i8)),
            Float(_) => return,
            Integer(n) => Float(n as f64),
            Null | Block(_, _) => Float(NAN),
            // TODO Return error
            Variable(_) => Float(NAN),
            Str(ref s) => Float(s.parse::<f64>().unwrap_or(NAN)),
        };
        *self = repl;
    }
    pub fn make_int(&mut self) {
        let repl = match *self {
            Bool(b) => Integer(b as i64),
            Integer(_) => return,
            Float(n) => Integer(n as i64),
            Null | Block(_, _) => Null,
            // TODO Return error
            Variable(_) => Null,
            Str(ref s) => s.parse::<i64>().map(Integer).unwrap_or(Null),
        };
        *self = repl;
    }
    pub fn flatten(self) -> Self {
        match self {
            Block(n, b) => {
                let len = n as usize * b.len();
                let b = b.into_iter().cycle().take(len).collect();

                Block(1, b)
            }
            a => a,
        }
    }
    pub fn pow(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Integer(a), Integer(b)) => Integer(a.pow(b as u32)),
            (Integer(a), Float(b)) => Float((a as f64).powf(b)),
            (Float(a), Integer(b)) => Float(a.powi(b as i32)),
            (Float(a), Float(b)) => Float(a.powf(b)),
            _ => Null,
        }
    }
}

impl From<bool> for Value {
    #[inline(always)]
    fn from(b: bool) -> Value {
        Bool(b)
    }
}
impl From<i64> for Value {
    #[inline(always)]
    fn from(i: i64) -> Value {
        Integer(i)
    }
}
impl From<f64> for Value {
    #[inline(always)]
    fn from(f: f64) -> Value {
        Float(f)
    }
}
impl From<&str> for Value {
    #[inline(always)]
    fn from(s: &str) -> Value {
        Str(s.to_owned())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Null, &Null) => true,
            (&Float(a), &Float(b)) => a == b,
            (&Integer(a), &Integer(b)) => a == b,
            (&Integer(a), &Float(b)) | (&Float(b), &Integer(a)) => a as f64 == b,
            (&Bool(a), &Bool(b)) => a == b,
            (&Str(ref a), &Str(ref b)) => a == b,
            (&Block(n, ref a), &Block(m, ref b)) => a == b && n == m,
            (&Variable(_), _) | (_, &Variable(_)) => false,
            _ => false
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Integer(a), Integer(b)) => a.partial_cmp(b),
            (&Integer(a), &Float(b)) | (&Float(b), &Integer(a)) => (a as f64).partial_cmp(&b),
            (Bool(a), Bool(b)) => a.partial_cmp(b),
            (Str(ref a), Str(ref b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

use crate::Value::*;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Float(ref n) => n.fmt(f),
            Integer(ref n) => n.fmt(f),
            Bool(ref n) => n.fmt(f),
            Str(ref s) => s.fmt(f),
            Block(_, _) => write!(f, "[code block]"),
            Variable(ref s) => write!(f, "[variable: {}]", s),
            Null => "NULL".fmt(f),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Float(ref n) => n.fmt(f),
            Integer(ref n) => n.fmt(f),
            Bool(ref n) => n.fmt(f),
            Str(ref s) => s.fmt(f),
            Null => write!(f, "null"),
            Block(n, ref b) => {
                let mut dbg = f.debug_set();
                for _ in 0..n {
                    dbg.entries(b);
                }
                dbg.finish()
            }
            Variable(ref s) => write!(f, "{}", s),
        }
    }
}

impl Not for Value {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Bool(b) => Bool(!b),
            Integer(n) => Integer(!n),
            Null | Block(_, _) | Variable(_) => Null,
            s @ Float(_) | s @ Str(_) => Bool(!s.as_bool()),
        }
    }
}

impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a & b),
            (Bool(a), Bool(b)) => Bool(a && b),
            (Block(_, _), _) | (_, Block(_, _)) => Null,
            (a, b) => (a.as_bool() && b.as_bool()).into(),
        }
    }
}

impl BitOr for Value {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a | b),
            (Bool(a), Bool(b)) => Bool(a || b),
            (Block(_, _), _) | (_, Block(_, _)) => Null,
            (a, b) => (a.as_bool() || b.as_bool()).into(),
        }
    }
}

impl BitXor for Value {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a ^ b),
            (Bool(a), Bool(b)) => Bool(a ^ b),
            (Block(_, _), _) | (_, Block(_, _)) => Null,
            (a, b) => (a.as_bool() ^ b.as_bool()).into(),
        }
    }
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Null, b) => b,
            (Str(a), b) => Str(format!("{}{}", a, b)),
            (a, Null) => a,
            (Float(a), Str(b)) => Str(format!("{}{}", a, b)),
            (Integer(a), Str(b)) => Str(format!("{}{}", a, b)),
            (Integer(a), Integer(b)) => Integer(a + b),
            (Integer(a), Float(b)) | (Float(b), Integer(a)) => Float(a as f64 + b),
            (Float(a), Float(b)) => Float(a + b),
            (Block(1, mut a), Block(1, b)) => {
                a.extend(b);
                Block(1, a)
            }
            (Block(an, a), Block(bn, b)) => {
                if a == b {
                    Block(an + bn, a)
                } else {
                    let mut res = Vec::with_capacity(an as usize * a.len() + bn as usize * b.len());

                    for _ in 0..an {
                        res.extend(a.iter().cloned())
                    }
                    for _ in 0..bn {
                        res.extend(b.iter().cloned())
                    }
                    Block(1, res)
                }
            }
            _ => Null,
        }
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Str(s), Integer(n)) | (Integer(n), Str(s)) => Str(s.repeat(n as usize)),
            (Integer(a), Integer(b)) => Integer(a * b),
            (Integer(a), Float(b)) | (Float(b), Integer(a)) => Float(a as f64 * b),
            (Float(a), Float(b)) => Float(a * b),
            (Integer(n), Block(bn, b)) | (Block(bn, b), Integer(n)) => Block(n as u16 * bn, b),
            _ => Null,
        }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a - b),
            (Float(a), Integer(b)) => Float(a - b as f64),
            (Integer(a), Float(b)) => Float(a as f64 - b),
            (Float(a), Float(b)) => Float(a - b),
            _ => Null,
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a / b),
            (Float(a), Integer(b)) => Float(a / b as f64),
            (Integer(a), Float(b)) => Float(a as f64 / b),
            (Float(a), Float(b)) => Float(a / b),
            _ => Null,
        }
    }
}

impl Rem for Value {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a % b),
            (Float(a), Integer(b)) => Float(a % b as f64),
            (Integer(a), Float(b)) => Float(a as f64 % b),
            (Float(a), Float(b)) => Float(a % b),
            _ => Null,
        }
    }
}
