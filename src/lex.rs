// Copyright 2021 Martin Pool

//! Lex text into tokens.

use crate::scan::Scan;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Plus,
    Minus,
    Star,
    Slash,
    Comma,
    Dot,
    Semicolon,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    True,
    False,

    String(String),
    Number(f64),
    Identifier(String),

    // keywords
    And,
    Class,
    Else,
    Fun,
    For, 
    If,
    Nil, 
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    /// 1-based source line where it occurs.
    pub line: usize,
    /// Literal content of the lexeme.
    pub lexeme: String,
}

pub struct Lexer<'s> {
    scan: Scan<char, std::str::Chars<'s>>,
    tokens: Vec<Token>,
}

impl<'s> Lexer<'s> {
    /// Construct a Lexer containing the tokens in the source.
    pub fn new(source: &str) -> Lexer {
        let mut lex = Lexer {
            scan: Scan::new(source.chars()),
            tokens: Vec::new(),
        };
        lex.lex();
        lex
    }

    /// Retrieve resulting tokens.
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    fn lex(&mut self) {
        while !self.scan.is_empty() {
            self.scan.start_token();
            let ch = self.scan.take().unwrap();
            let tok = match ch {
                '\n' | ' ' | '\t' | '\r' => {
                    continue;
                }
                '+' => Tok::Plus,
                '*' => Tok::Star,
                '-' => Tok::Minus,
                '.' => Tok::Dot,
                '/' if self.scan.peek() == Some(&'/') => {
                    self.scan.take_until(|cc| *cc == '\n');
                    continue; // drop the comment
                }
                '/' => Tok::Slash,
                '0'..='9' => self.number(),
                '{' => Tok::LeftBrace,
                '}' => Tok::RightBrace,
                '(' => Tok::LeftParen,
                ')' => Tok::RightParen,
                '"' => self.string(),
                ch if ch.is_ascii_alphabetic() => self.word(),
                '_' => self.word(),
                other => panic!("unhandled character {:?}", other),
            };
            self.tokens.push(Token {
                tok,
                lexeme: self.scan.current_token(),
                line: self.scan.current_token_start_line(),
            });
        }
    }

    fn number(&mut self) -> Tok {
        self.scan.take_while(|c| c.is_ascii_digit());
        match self.scan.peek2() {
            Some(('.', cc)) if cc.is_ascii_digit() => {
                self.scan.take_exactly(&'.').unwrap();
                self.scan.take_while(|c| c.is_ascii_digit());
            }
            _ => (),
        }
        let val: f64 = self.scan.current_token::<String>().parse().unwrap();
        Tok::Number(val)
    }

    fn string(&mut self) -> Tok {
        // TODO: Handle backslash escapes.
        // TODO: Clean error if the string is unterminated.
        let mut s = String::new();
        while let Some(c) = self.scan.take_if(|c| *c != '"') {
            s.push(c)
        }
        if self.scan.take_exactly(&'"').is_none() {
            panic!("unterminated string");
        }
        Tok::String(s)
    }

    fn word(&mut self) -> Tok {
        self.scan
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_');
        let s: String = self.scan.current_token();
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

    fn lex_tokens(s: &str) -> Vec<Token> {
        Lexer::new(s).tokens().to_vec()
    }

    fn lex_toks<'s>(s: &'s str) -> Vec<Tok> {
        Lexer::new(s)
            .tokens()
            .iter()
            .map(|t| t.tok.clone())
            .collect()
    }

    #[test]
    fn can_scan_integer() {
        assert_eq!(
            lex_tokens("12345"),
            &[Token {
                tok: Tok::Number(12345.0),
                line: 1,
                lexeme: "12345".to_owned(),
            }],
        );
    }

    #[test]
    fn integer_followed_by_dot_is_not_float() {
        assert_eq!(lex_toks("1234."), vec![Tok::Number(1234.0), Tok::Dot,]);
    }

    #[test]
    fn decimal_float() {
        assert_eq!(lex_toks("3.1415"), vec![Tok::Number(3.1415),]);
    }

    #[test]
    fn skip_comments() {
        assert_eq!(
            lex_tokens("1\n// two would be here\n\n3.000\n\n// the end!\n"),
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
        assert_eq!(lex_tokens("// nothing else, not even a newline"), vec![]);
    }

    #[test]
    fn just_some_comments() {
        assert_eq!(lex_tokens("// a comment\n\n\n// then another\n"), vec![]);
    }

    #[test]
    fn simple_string() {
        assert_eq!(
            lex_tokens(r#""hello Lox?""#),
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
            lex_tokens(src),
            vec![Token {
                tok: Tok::String("one\nokapi\ntwo\n".to_owned()),
                line: 1,
                lexeme: src.to_owned(),
            }]
        );
    }

    #[should_panic]
    #[test]
    fn unterminated_string_errors() {
        let src = "\"going along...";
        // TODO: Give a nice error rather than panic
        let _v = lex_tokens(src);
    }

    #[test]
    fn words_and_keywords() {
        let src = "true false maybe __secret__";
        assert_eq!(
            lex_toks(src),
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
            lex_toks(src),
            [Tok::Plus, Tok::Minus, Tok::Star, Tok::Slash]
        );
    }
}
