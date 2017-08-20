#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    Value(String),
    BeginBlock,
    EndBlock,
    EmptyBlock,
    True,
    False,
    NullVal,
    Dup,
    Not,
    If,
    ApplyFunction,
    Read,
    Swap,
    Drop,
    CastNum,
    Eq,
    Neq,
    Write,
    Print,
    Or,
    And,
    Add,
    Sub,
    Mul,
    Div,
    Rem
}

use self::Command::*;

impl Command {
    pub fn from_str(cmd: &str) -> Self {
        match &*cmd.to_lowercase() {
            "{"|"["|"do" => BeginBlock,
            "}"|"]"|"end" => EndBlock,
            "{}"|"[]"|"nop" => EmptyBlock,
            "t" | "true" => True,
            "f" | "false" => False,
            "¤" | "null" | "nil" => NullVal,
            ";" | "dup" => Dup,
            "!" | "not" => Not,
            "?" | "if" => If,
            "()" => ApplyFunction,
            "<" | "read" => Read,
            "$" | "swap" | "exch" => Swap,
            "drop" => Drop,
            "#" | "num" => CastNum,
            "==" | "=" | "eq" => Eq,
            "!=" | "~=" | "neq" => Neq,
            ">" | "wrte" => Write,
            "_" | "prnt" => Print,
            "||" | "|" | "or" => Or,
            "&&" | "&" | "and" => And,
            "+" | "add" => Add,
            "-" | "sub" => Sub,
            "*" | "mul" => Mul,
            "/" | "div" => Div,
            "%" | "rem" => Rem,
            _ => Value(cmd.to_owned())
        }
    }
}
