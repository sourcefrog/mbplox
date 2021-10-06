// Copyright 2021 Martin Pool

//! Parse a stream of tokens into an AST.

// use anyhow::Result;

use crate::ast::Expr;
use crate::lex::{Tok, Token};
use crate::value::Value;

// General approach to the parser API:
//
// At every point of trying to parse something, it seems like
// we could either
// - succeed and parse the expected value
// - not see any more of them
// - get an error
//
// However the distinction between "we didn't see one" and "there
// was an error" may not be clear, and maybe should be made at a
// different level?
//
// This is intended to be in the parser combinator style, written
// from scratch as a learning exercise...

pub fn parse_literal(tokens: &[Token]) -> Option<(Expr, &[Token])> {
    if let Some(first) = tokens.first() {
        if let Some(value) = Value::from_literal_token(first) {
            Some((Expr::Literal(value), &tokens[1..]))
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lex::lex;

    /// Parse a string, expecting that there are no errors and nothing
    /// remaining unparsed.
    fn parse_exactly(source: &str, parse_fn: fn(&[Token]) -> Option<(Expr, &[Token])>) -> Expr {
        let (tokens, errs) = lex(source);
        assert_eq!(errs.len(), 0);
        let (expr, remaining) = parse_fn(&tokens).unwrap();
        assert_eq!(remaining.len(), 0);
        expr
    }

    #[test]
    fn parse_literal_number() {
        assert_eq!(
            parse_exactly("69\n", parse_literal),
            Expr::Literal(Value::Number(69.0))
        );
    }

    #[test]
    fn parse_literal_nil() {
        assert_eq!(
            parse_exactly("nil\n", parse_literal),
            Expr::Literal(Value::Nil)
        );
    }

    #[test]
    fn parse_literal_string() {
        assert_eq!(
            parse_exactly("\"69\"\n", parse_literal),
            Expr::Literal(Value::String("69".to_owned()))
        );
    }

    #[test]
    fn parse_literal_false() {
        assert_eq!(
            parse_exactly("\nfalse\n", parse_literal),
            Expr::Literal(Value::Bool(false))
        );
    }

    #[test]
    fn parse_literal_true() {
        assert_eq!(
            parse_exactly("\ntrue\n", parse_literal),
            Expr::Literal(Value::Bool(true))
        );
    }
}
