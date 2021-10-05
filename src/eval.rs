use anyhow::{anyhow, Result};

use crate::ast;
use crate::value::Value;

pub struct Interpreter {}

pub enum Error {}

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
