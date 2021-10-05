// Copyright 2021 Martin Pool

//! Representable Lox values.

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
}
