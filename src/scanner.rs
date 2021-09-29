// Copyright 2021 Martin Pool

//! Scan text to tokens.

// use crate::Result;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,

    // Literals
    // String(String),
    Number(f64),
    // Identifier(String),
    Eof,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    token_type: TokenType,
    /// 1-based source line where it occurs.
    line: usize,
}

/// Return an iterator over the tokens in the source.
pub fn scan<'s>(source: &'s str) -> impl Iterator<Item = Token> + 's {
    Scanner {
        // source,
        chars: Peek::new(source.chars()),
        line: 1,
        past_eof: false,
    }
}

struct Scanner<'s> {
    chars: Peek<char, std::str::Chars<'s>>,
    // source: &'s str,
    line: usize,
    past_eof: bool,
}

impl<'s> Iterator for Scanner<'s> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.past_eof {
                return None;
            } else if self.chars.is_empty() {
                // Return one Eof and then stop the iterator.
                // (Maybe it should just stop?)
                self.past_eof = true;
                return self.emit(TokenType::Eof);
            }
            let ch = self.chars.take().unwrap();

            let token_type = match ch {
                '\n' => {
                    self.line += 1;
                    continue;
                }
                ' ' | '\t' | '\r' => {
                    continue;
                }
                '+' => TokenType::Plus,
                '*' => TokenType::Star,
                '-' => TokenType::Minus,
                '0'..='9' => {
                    let mut s = String::new();
                    s.push(ch);
                    while let Some(cc) = self.chars.take_if(|c| c.is_ascii_digit()) {
                        s.push(cc)
                    }
                    if self.chars.take_if(|&c| c == '.').is_some() {
                        s.push('.');
                        while let Some(cc) = self.chars.take_if(|c| c.is_ascii_digit()) {
                            s.push(cc)
                        }
                    }
                    let val: f64 = s.parse().unwrap();
                    TokenType::Number(val)
                }
                other => panic!("unhandled character {:?}", other),
            };
            return self.emit(token_type);
        }
    }
}

impl<'s> Scanner<'s> {
    fn emit(&self, token_type: TokenType) -> Option<Token> {
        Some(Token {
            token_type,
            line: self.line,
        })
    }
}

/// Iterator adapter allowing arbitrary-length peeking ahead.
///
/// Beyond [std::iter::Peekable] this allows looking more than one
/// item ahead.
struct Peek<C, I>
where
    I: Iterator<Item = C>,
{
    inner: I,
    buf: Vec<C>,
}

impl<C, I> Peek<C, I>
where
    I: Iterator<Item = C>,
{
    fn new(inner: I) -> Peek<C, I> {
        Peek {
            inner,
            buf: Vec::new(),
        }
    }

    fn take(&mut self) -> Option<C> {
        if self.buf.is_empty() {
            self.inner.next()
        } else {
            Some(self.buf.remove(0))
        }
    }

    fn take_if<F>(&mut self, f: F) -> Option<C>
    where
        F: FnOnce(&C) -> bool,
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

    fn peek(&mut self) -> Option<&C> {
        if self.buf.is_empty() {
            if let Some(c) = self.inner.next() {
                self.buf.push(c)
            } else {
                return None;
            }
        }
        Some(&self.buf[0])
    }

    fn is_empty(&mut self) -> bool {
        !self.peek().is_some()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_scan_integer() {
        itertools::assert_equal(
            scan("12345"),
            [
                Token {
                    token_type: TokenType::Number(12345.0),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Eof,
                    line: 1,
                },
            ],
        );
    }
}
