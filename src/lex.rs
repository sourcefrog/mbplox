// Copyright 2021 Martin Pool

//! Lex text into tokens.

use crate::scan::Scan;

#[derive(Debug, PartialEq)]
pub enum Tok {
    Plus,
    Minus,
    Star,
    Slash,
    Dot,

    True,
    False,

    // Literals
    String(String),
    Number(f64),
    Identifier(String),
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
        chars: Scan::new(source.chars()),
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
                '/' => Tok::Slash,
                '0'..='9' => self.number(),
                '"' => self.string(),
                ch if ch.is_ascii_alphabetic() => self.word(),
                '_' => self.word(),
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
            line: self.chars.current_token_start_line(),
        })
    }

    fn number(&mut self) -> Tok {
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
        Tok::Number(val)
    }

    fn string(&mut self) -> Tok {
        // TODO: Handle backslash escapes.
        // TODO: Error if the string is unterminated.
        self.chars.take_until(|c| *c == '"');
        // Omit the starting and ending quotes
        let l = self.chars.current_token().count();
        let s: String = self.chars.current_token().skip(1).take(l - 2).collect();
        Tok::String(s)
    }

    fn word(&mut self) -> Tok {
        self.chars
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_');
        let s: String = self.chars.current_token().collect();
        match s.as_str() {
            "true" => Tok::True,
            "false" => Tok::False,
            _ => Tok::Identifier(s),
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

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

    #[test]
    fn simple_string() {
        assert_eq!(
            lex(r#""hello Lox?""#).collect::<Vec<Token>>(),
            vec![Token {
                tok: Tok::String("hello Lox?".to_owned()),
                line: 1,
                lexeme: r#""hello Lox?""#.to_owned(),
            }]
        );
    }

    #[test]
    fn multi_line_string_has_line_number_of_start() {
        let src = "\"one\nokapi\ntwo\n\"";
        assert_eq!(
            lex(src).collect::<Vec<Token>>(),
            vec![Token {
                tok: Tok::String("one\nokapi\ntwo\n".to_owned()),
                line: 1,
                lexeme: src.to_owned(),
            }]
        );
    }

    #[test]
    fn words_and_keywords() {
        let src = "true false maybe __secret__";
        assert_eq!(
            lex(src).map(|token| token.tok).collect::<Vec<Tok>>(),
            [
                Tok::True,
                Tok::False,
                Tok::Identifier("maybe".to_owned()),
                Tok::Identifier("__secret__".to_owned())
            ]
        );
    }

    #[test]
    fn operators() {
        let src = "+-*/";
        assert_eq!(
            lex(src).map(|token| token.tok).collect::<Vec<Tok>>(),
            [Tok::Plus, Tok::Minus, Tok::Star, Tok::Slash]
        );
    }
}
