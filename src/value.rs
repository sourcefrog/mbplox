// Copyright 2021 Martin Pool

//! Representable Lox values.

use std::fmt;

use crate::lex::{Tok, Token};

/// Any type of Lox value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    String(String),
    Number(f64),
}

impl Value {
    pub fn not(&self) -> Value {
        use Value::*;
        match self {
            Nil => Nil,
            Bool(b) => Bool(!b),
            Number(_) | String(_) => unimplemented!(),
        }
    }

    pub fn from_literal_token(token: &Token) -> Option<Value> {
        match &token.tok {
            Tok::Number(n) => Some(Value::Number(*n)),
            Tok::String(s) => Some(Value::String(s.clone())),
            Tok::True => Some(Value::Bool(true)),
            Tok::False => Some(Value::Bool(false)),
            Tok::Nil => Some(Value::Nil),
            _ => None,
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(f)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Value;

    #[test]
    fn display_value() {
        let cases = [
            (Value::Nil, "nil"),
            (Value::Bool(true), "true"),
            (Value::Bool(false), "false"),
            (Value::Number(3.14156), "3.14156"),
            (Value::from(747.), "747"),
            (Value::from(""), ""),
            (Value::from("747."), "747."),
            (Value::from("hello\nworld\n"), "hello\nworld\n"),
        ];
        for (value, expected) in cases {
            assert_eq!(format!("{}", value), expected);
        }
    }
}
