// Copyright 2021 Martin Pool

//! Representable Lox values.

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
