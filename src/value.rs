use std::ops::*;
use std::fmt;

use cmd::Command;

#[derive(Clone)]
pub enum Value {
    Num(f64),
    Str(String),
    Variable(String),
    Block(u16, Vec<Command>),
    Null
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match *self {
            Num(n) => !n.is_nan() && n != 0.,
            Str(ref s) => !s.is_empty(),
            Block(_, _) => true,
            // TODO Return error
            Variable(_) => false,
            Null => false
        }
    }
    pub fn make_num(&mut self) {
        let repl = match *self {
            Num(_) => return,
            Null | Block(_, _) => Num(0./0.),
            // TODO Return error
            Variable(_) => Num(0./0.),
            Str(ref s) => {
                if s == "true" {
                    Num(1.)
                } else if s == "false" {
                    Num(0.)
                } else {
                    Num(s.parse::<f64>().unwrap_or(0./0.))
                }
            }
        };
        *self = repl;
    }
}

impl From<bool> for Value {
    #[inline(always)]
    fn from(b: bool) -> Value {
        match b {
            true => Num(1.),
            false => Num(0.),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Null, &Null) => true,
            (&Num(a), &Num(b)) => a == b,
            (&Str(ref a), &Str(ref b)) => a == b,
            _ => false
        }
    }
}

use Value::*;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Num(ref n) => n.fmt(f),
            Str(ref s) => s.fmt(f),
            Block(_, _) => write!(f, "[code block]"),
            Variable(ref s)=> write!(f, "[variable: {}]", s),
            Null       => "NULL".fmt(f)
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Num(ref n) => n.fmt(f),
            Str(ref s) => s.fmt(f),
            Null => write!(f, "Null"),
            Block(n, ref b) => {
                write!(f, "Block")?;
                let mut dbg = f.debug_set();
                for _ in 0..n {
                    dbg.entries(b);
                }
                dbg.finish()
            },
            Variable(ref s)=> f.debug_tuple("VarName").field(s).finish(),
        }
    }
}

impl Not for Value {
    type Output = Self;
    fn not(self) -> Self {
        self.as_bool().into()
    }
}

impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (Num(a), Num(b)) => Num((a as i64 & b as i64) as f64),
            (Block(_, _), _) | (_, Block(_, _)) => Null,
            (a, b)  => (a.as_bool() && b.as_bool()).into(),
        }
    }
}

impl BitOr for Value {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        match (self, other) {
            (Num(a), Num(b)) => Num((a as i64 | b as i64) as f64),
            (Block(_, _), _) | (_, Block(_, _)) => Null,
            (a, b)  => (a.as_bool() || b.as_bool()).into(),
        }
    }
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Null, b) => b,
            (Str(a), b)  => Str(format!("{}{}", a, b)),
            (a, Null) => a,
            (Num(a), Str(b))  => Str(format!("{}{}", a, b)),
            (Num(a), Num(b)) => Num(a + b),
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
            (Str(s), Num(n)) | (Num(n), Str(s))  => Str(s.repeat(n as usize)),
            (Num(a), Num(b)) => Num(a * b),
            (Num(n), Block(bn, b)) | (Block(bn, b), Num(n)) => {
                Block(n as u16 * bn, b)
            }
            _ => Null,
        }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Num(a), Num(b)) => Num(a - b),
            _ => Null,
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Num(a), Num(b)) => Num(a / b),
            _ => Null,
        }
    }
}

impl Rem for Value {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Num(a), Num(b)) => Num(a % b),
            _ => Null,
        }
    }
}
