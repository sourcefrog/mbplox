// Copyright 2021 Martin Pool

//! Representable Lox values.

/// Any type of Lox value.
#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    String(String),
    Number(f64),
}
