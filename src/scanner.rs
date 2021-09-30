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
pub fn scan(source: &str) -> impl Iterator<Item = Token> + '_ {
    Scanner {
        chars: Peek::new(source.chars()),
        line: 1,
    }
}

struct Scanner<'s> {
    chars: Peek<char, std::str::Chars<'s>>,
    // source: &'s str,
    line: usize,
}

impl<'s> Iterator for Scanner<'s> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.chars.is_empty() {
                return None;
            }
            let ch = self.chars.take().unwrap();
            // Maybe collecting the lexeme should be integrated with `Peek`,
            // with some kind of reset at the start of the token?
            let lexeme = String::from(ch);

            let token_type = match ch {
                '\n' => {
                    self.line += 1;
                    continue;
                }
                ' ' | '\t' | '\r' => {
                    continue;
                }
                '+' => Tok::Plus,
                '*' => Tok::Star,
                '-' => Tok::Minus,
                '.' => Tok::Dot,
                '/' if self.chars.peek() == Some(&'/') => {
                    while let Some(cc) = self.chars.take() {
                        if cc == '\n' {
                            self.line += 1;
                            break;
                        }
                    }
                    continue; // drop the comment
                }
                '/' => Tok::Dot,
                '0'..='9' => return self.number(ch),
                other => panic!("unhandled character {:?}", other),
            };
            return self.make_token(token_type, lexeme);
        }
    }
}

impl<'s> Scanner<'s> {
    fn make_token(&self, token_type: Tok, lexeme: String) -> Option<Token> {
        Some(Token {
            tok: token_type,
            lexeme,
            line: self.line,
        })
    }

    fn number(&mut self, first_char: char) -> Option<Token> {
        let mut s = String::from(first_char);
        while let Some(cc) = self.chars.take_if(|c| c.is_ascii_digit()) {
            s.push(cc)
        }
        if self.chars.peek() == Some(&'.')
            && self
                .chars
                .peek_nth(1)
                .map(|cc| cc.is_ascii_digit())
                .unwrap_or_default()
        {
            self.chars.take_exactly(&'.').unwrap();
            s.push('.');
            while let Some(cc) = self.chars.take_if(|c| c.is_ascii_digit()) {
                s.push(cc)
            }
        }
        let val: f64 = s.parse().unwrap();

        self.make_token(Tok::Number(val), s)
    }
}

/// Iterator adapter allowing arbitrary-length peeking ahead.
///
/// Beyond [std::iter::Peekable] this allows looking more than one
/// item ahead.
struct Peek<C, I>
where
    I: Iterator<Item = C>,
    C: PartialEq,
{
    inner: I,
    buf: Vec<C>,
}

impl<C, I> Peek<C, I>
where
    I: Iterator<Item = C>,
    C: PartialEq,
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

    fn take_exactly(&mut self, c: &C) -> Option<C> {
        self.take_if(|cc| *cc == *c)
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
        self.peek_nth(0)
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
            scan("12345"),
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
            scan("1234.")
                .map(|t| t.tok)
                .collect::<Vec<Tok>>(),
            vec![Tok::Number(1234.0), Tok::Dot,]
        );
    }

    #[test]
    fn decimal_float() {
        assert_eq!(
            scan("3.1415")
                .map(|t| t.tok)
                .collect::<Vec<Tok>>(),
            vec![Tok::Number(3.1415),]
        );
    }

    #[test]
    fn skip_comments() {
        assert_eq!(
            scan("1\n// two would be here\n\n3.000\n\n// the end!\n").collect::<Vec<Token>>(),
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
            scan("// nothing else, not even a newline").collect::<Vec<Token>>(),
            vec![]
        );
    }

    #[test]
    fn just_some_comments() {
        assert_eq!(
            scan("// a comment\n\n\n// then another\n").collect::<Vec<Token>>(),
            vec![]
        );
    }
}
