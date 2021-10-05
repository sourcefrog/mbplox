// Copyright 2021 Martin Pool

use crate::value::Value;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Value),
    Grouping {
        expr: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Negative,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    EqualEqual,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
}
