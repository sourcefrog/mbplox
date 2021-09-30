// Copyright 2021 Martin Pool

//! Scan text to tokens.

// use crate::Result;

#[derive(Debug, PartialEq)]
pub enum Tok {
    Plus,
    Minus,
    Star,
    Dot,

    // Literals
    // String(String),
    Number(f64),
    // Identifier(String),
}

#[derive(Debug, PartialEq)]
pub struct Token {
    tok: Tok,
    /// 1-based source line where it occurs.
    line: usize,
    /// Literal content of the lexeme.
    lexeme: String,
}

/// Return an iterator over the tokens in the source.
pub fn lex(source: &str) -> impl Iterator<Item = Token> + '_ {
    Lexer {
        chars: Scan::new(source.chars(), |c: &char| *c == '\n'),
    }
}

struct Lexer<'s> {
    chars: Scan<char, std::str::Chars<'s>>,
}

impl<'s> Iterator for Lexer<'s> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.chars.is_empty() {
                return None;
            }
            self.chars.start_token();
            let ch = self.chars.take().unwrap();
            let token_type = match ch {
                '\n' | ' ' | '\t' | '\r' => {
                    continue;
                }
                '+' => Tok::Plus,
                '*' => Tok::Star,
                '-' => Tok::Minus,
                '.' => Tok::Dot,
                '/' if self.chars.peek() == Some(&'/') => {
                    self.chars.take_until(|cc| *cc == '\n');
                    continue; // drop the comment
                }
                '/' => Tok::Dot,
                '0'..='9' => return self.number(),
                other => panic!("unhandled character {:?}", other),
            };
            return self.make_token(token_type);
        }
    }
}

impl<'s> Lexer<'s> {
    fn make_token(&self, tok: Tok) -> Option<Token> {
        Some(Token {
            tok,
            lexeme: self.chars.current_token().collect(),
            line: self.chars.line_number(),
        })
    }

    fn number(&mut self) -> Option<Token> {
        self.chars.take_while(|c| c.is_ascii_digit());
        match self.chars.peek2() {
            Some(('.', cc)) if cc.is_ascii_digit() => {
                self.chars.take_exactly(&'.').unwrap();
                self.chars.take_while(|c| c.is_ascii_digit());
            }
            _ => (),
        }
        let val: f64 = self
            .chars
            .current_token()
            .collect::<String>()
            .parse()
            .unwrap();
        self.make_token(Tok::Number(val))
    }
}

/// Iterator adapter allowing arbitrary-length peeking ahead.
///
/// Provides low-level char parsing without knowing anything specific about the
/// grammar.
///
/// Beyond [std::iter::Peekable] this allows looking more than one item ahead.
struct Scan<C, I>
where
    I: Iterator<Item = C>,
    C: PartialEq + Clone,
{
    inner: I,
    buf: Vec<C>,
    current_token: Vec<C>,
    is_newline: fn(&C) -> bool,
    line_number: usize,
}

impl<C, I> Scan<C, I>
where
    I: Iterator<Item = C>,
    C: PartialEq + Clone,
{
    fn new(inner: I, is_newline: fn(&C) -> bool) -> Scan<C, I> {
        Scan {
            inner,
            buf: Vec::new(),
            current_token: Vec::new(),
            is_newline,
            line_number: 1,
        }
    }

    fn start_token(&mut self) {
        self.current_token.clear()
    }

    /// Return all the atoms recognized since the last [start_token].
    fn current_token(&self) -> impl Iterator<Item = &C> {
        self.current_token.iter()
    }

    /// Consume and return one atom.
    ///
    /// All consumption should go through here to maintain invariants, including
    /// line numbering and accumulating the current token.
    fn take(&mut self) -> Option<C> {
        let c = if self.buf.is_empty() {
            self.inner.next()?
        } else {
            self.buf.remove(0)
        };
        if (self.is_newline)(&c) {
            self.line_number += 1;
        }
        self.current_token.push(c.clone());
        Some(c)
    }

    fn line_number(&self) -> usize {
        self.line_number
    }

    fn take_if<F>(&mut self, f: F) -> Option<C>
    where
        F: Fn(&C) -> bool,
    {
        match self.peek() {
            None => None,
            Some(c) => {
                if f(c) {
                    self.take()
                } else {
                    None
                }
            }
        }
    }

    pub fn take_while(&mut self, f: fn(&C) -> bool) {
        while self.take_if(f).is_some() {}
    }

    /// Take characters up to and including a terminator.
    pub fn take_until(&mut self, f: fn(&C) -> bool) {
        while let Some(c) = self.take() {
            if f(&c) {
                break;
            }
        }
    }

    fn take_exactly(&mut self, c: &C) -> Option<C> {
        self.take_if(|cc| *cc == *c)
    }

    fn peek(&mut self) -> Option<&C> {
        self.peek_nth(0)
    }

    fn peek2(&mut self) -> Option<(&C, &C)> {
        if self.peek_nth(1).is_some() {
            Some((&self.buf[0], &self.buf[1]))
        } else {
            None
        }
    }

    fn peek_nth(&mut self, n: usize) -> Option<&C> {
        while self.buf.len() <= n {
            if let Some(c) = self.inner.next() {
                self.buf.push(c)
            } else {
                return None;
            }
        }
        Some(&self.buf[n])
    }

    fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_scan_integer() {
        itertools::assert_equal(
            lex("12345"),
            [Token {
                tok: Tok::Number(12345.0),
                line: 1,
                lexeme: "12345".to_owned(),
            }],
        );
    }

    #[test]
    fn integer_followed_by_dot_is_not_float() {
        assert_eq!(
            lex("1234.").map(|t| t.tok).collect::<Vec<Tok>>(),
            vec![Tok::Number(1234.0), Tok::Dot,]
        );
    }

    #[test]
    fn decimal_float() {
        assert_eq!(
            lex("3.1415").map(|t| t.tok).collect::<Vec<Tok>>(),
            vec![Tok::Number(3.1415),]
        );
    }

    #[test]
    fn skip_comments() {
        assert_eq!(
            lex("1\n// two would be here\n\n3.000\n\n// the end!\n").collect::<Vec<Token>>(),
            vec![
                Token {
                    tok: Tok::Number(1.0),
                    line: 1,
                    lexeme: "1".to_owned(),
                },
                Token {
                    tok: Tok::Number(3.0),
                    line: 4,
                    lexeme: "3.000".to_owned()
                },
            ]
        );
    }

    #[test]
    fn just_a_comment() {
        assert_eq!(
            lex("// nothing else, not even a newline").collect::<Vec<Token>>(),
            vec![]
        );
    }

    #[test]
    fn just_some_comments() {
        assert_eq!(
            lex("// a comment\n\n\n// then another\n").collect::<Vec<Token>>(),
            vec![]
        );
    }
}
