// Copyright 2021 Martin Pool

//! Scan text to tokens.

use std::iter::Peekable;

use crate::Result;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,

    // Literals
    String(String),
    Number(f64),
    Identifier(String),

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
        chars: source.chars().peekable(),
        line: 1,
        past_eof: false,
    }
}

struct Scanner<'s> {
    chars: Peekable<std::str::Chars<'s>>,
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
            } else if let Some(ch) = self.chars.next() {
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
                        while let Some(cc) = self.take_if(|c| c.is_ascii_digit()) {
                            s.push(cc)
                        }
                        if self.take_if(|c| c == '.').is_some() {
                            s.push('.');
                            while let Some(cc) = self.take_if(|c| c.is_ascii_digit()) {
                                s.push(cc)
                            }
                        }
                        let val: f64 = s.parse().unwrap();
                        TokenType::Number(val)
                    }
                    other => panic!("unhandled character {:?}", other),
                };
                return self.emit(token_type);
            } else {
                // Return one Eof and then stop the iterator.
                // (Maybe it should just stop?)
                self.past_eof = true;
                return self.emit(TokenType::Eof);
            }
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

    fn take_if<F>(&mut self, f: F) -> Option<char>
    where
        F: FnOnce(char) -> bool,
    {
        match self.chars.peek() {
            None => None,
            Some(&c) => {
                if f(c) {
                    self.chars.next();
                    Some(c)
                } else {
                    None
                }
            }
        }
    }
}
