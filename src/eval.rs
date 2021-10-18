// Copyright 2021 Martin Pool

//! Evaluate Lox source.

use anyhow::{anyhow, Result};

use crate::ast;
use crate::lex::{lex, Token};
use crate::parse;
use crate::value::Value;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn eval(&mut self, source: &str) -> Result<Value> {
        let results = lex(source);
        dbg!(&results);
        // TODO: Print all errors; return the first one (or all of them?).
        let tokens: Vec<Token> = results.into_iter().map(Result::unwrap).collect();

        let (expr, rest) = parse::parse_expr(&tokens)?;
        dbg!(&expr);
        assert!(rest.is_empty());

        let value = expr.eval()?;
        dbg!(&value);

        Ok(value)
    }
}

pub trait Eval {
    fn eval(&self) -> Result<Value>;
}

impl Eval for ast::Expr {
    fn eval(&self) -> Result<Value> {
        use ast::Expr::*;
        match self {
            Literal(value) => Ok(value.clone()),
            Grouping { expr } => expr.eval(),
            Unary { op, expr } => apply_unary(op, expr.eval()?),
            _other => unimplemented!(),
        }
    }
}

fn apply_unary(op: &ast::UnaryOp, value: Value) -> Result<Value> {
    match op {
        ast::UnaryOp::Not => Ok(value.not()),
        _other => Err(anyhow!("{:?} not implemented", op)),
    }
}

#[cfg(test)]
mod test {
    use super::Interpreter;
    use crate::value::Value;

    #[test]
    fn eval_literal_integer() {
        assert_eq!(
            Interpreter::new().eval("1234").unwrap(),
            Value::Number(1234.0)
        );
    }
}
