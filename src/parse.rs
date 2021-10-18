// Copyright 2021 Martin Pool

//! Parse a stream of tokens into an AST.

use anyhow::{anyhow, Result};

use crate::ast::Expr;
use crate::lex::Token;
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

/// Parse a literal value: string, number, bool, or nil.
fn parse_literal(tokens: &[Token]) -> Result<(Expr, &[Token])> {
    take_if(tokens, |t| Value::from_literal_token(t).map(Expr::Literal))
        .ok_or(anyhow!("not a literal"))
}

///// Parse a unary expression:
/////
///// unary          → ( "-" | "!" ) expression ;
//fn parse_unary(_tokens: &[Token]) -> Result<(Expr, &[Token])> {
//    todo!()
//}

/// Parse any expression
///
///    expression     → literal
///                   | unary
///                   | binary
///                   | grouping

pub fn parse_expr(tokens: &[Token]) -> Result<(Expr, &[Token])> {
    let (expr, rest) = parse_literal(tokens)?;
    if let Some(next_token) = rest.first() {
        return Err(anyhow!(
            "unexpected tokens after literal {:?}: {:?}",
            expr,
            next_token
        ));
    }
    Ok((expr, rest))
}

/// Parse and consume one element if the function matches it.
fn take_if<T>(tokens: &[Token], match_fn: fn(&Token) -> Option<T>) -> Option<(T, &[Token])> {
    tokens.first().and_then(match_fn).map(|t| (t, &tokens[1..]))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lex::lex;

    /// Parse a string, expecting that there are no errors and nothing
    /// remaining unparsed.
    fn parse_exactly(source: &str, parse_fn: fn(&[Token]) -> Result<(Expr, &[Token])>) -> Expr {
        let tokens = lex(source)
            .into_iter()
            .map(Result::unwrap)
            .collect::<Vec<Token>>();
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
