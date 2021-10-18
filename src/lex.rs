// Copyright 2021 Martin Pool

//! Lex text into tokens.
//!
//! This is the lower level of parsing.

use std::fmt;

use crate::place::Place;
use crate::scan::Scan;

/// A specific type of lexical tokens, including the embedded value of literals, and the identifier
/// string for identifiers.
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
    While,
}

/// A lexical token.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    /// Place where this token starts.
    pub place: Place,
    /// Literal content of the lexeme.
    // TODO: Is the lexeme ever really needed?
    pub lexeme: String,
}

/// An error while tokenizing source.
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    /// Place in the source where the error occurred.
    pub place: Place,
    /// Type of lexer error.
    pub kind: ErrorKind,
}

impl fmt::Display for Error {
    // TODO: Maybe move this to a common error-printing trait across all error classes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] Error: {}.", self.place, self.kind)
    }
}

/// A specific kind of tokenization error.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// A character that just can't occur in mbplox.
    UnexpectedCharacter(char),
    /// A double-quoted string was still open at the end of the file.
    UnterminatedString,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        match self {
            UnexpectedCharacter(ch) => write!(f, "unexpected character {:?}", ch),
            UnterminatedString => write!(f, "unterminated string"),
        }
    }
}

/// Lex some Lox source into a vec of tokens and tokenization errors.
pub fn lex(source: &str) -> Vec<Result<Token, Error>> {
    let mut scan = Scan::new(source);
    let mut result = Vec::new();
    while !scan.is_empty() {
        scan.start_token();
        let tok = match scan.take().unwrap() {
            '\n' | ' ' | '\t' | '\r' => {
                continue;
            }
            '+' => Tok::Plus,
            '*' => Tok::Star,
            '-' => Tok::Minus,
            '.' => Tok::Dot,
            '/' if scan.take_exactly('/') => {
                scan.take_until(|cc| *cc == '\n');
                continue; // drop the comment
            }
            '/' => Tok::Slash,
            ';' => Tok::Semicolon,
            ',' => Tok::Comma,
            '!' if scan.take_exactly('=') => Tok::BangEqual,
            '!' => Tok::Bang,
            '=' if scan.take_exactly('=') => Tok::EqualEqual,
            '=' => Tok::Equal,
            '0'..='9' => number(&mut scan),
            '{' => Tok::LeftBrace,
            '}' => Tok::RightBrace,
            '(' => Tok::LeftParen,
            ')' => Tok::RightParen,
            '<' if scan.take_exactly('=') => Tok::LessEqual,
            '<' => Tok::Less,
            '>' if scan.take_exactly('=') => Tok::GreaterEqual,
            '>' => Tok::Greater,
            '"' => {
                result.push(string(&mut scan));
                continue;
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => word(&mut scan),
            '#' if scan.next_column() == 2 && scan.take_exactly('!') => {
                // drop shebang line
                scan.take_until(|cc| *cc == '\n');
                continue;
            }
            other => {
                result.push(Err(Error {
                    place: scan.token_start(),
                    kind: ErrorKind::UnexpectedCharacter(other),
                }));
                continue;
            }
        };
        result.push(Ok(Token {
            tok,
            lexeme: scan.current_token().to_owned(),
            place: scan.token_start(),
        }));
    }
    result
}

fn number(scan: &mut Scan) -> Tok {
    scan.take_while(|c| c.is_ascii_digit());
    match scan.peek2() {
        Some(('.', cc)) if cc.is_ascii_digit() => {
            debug_assert!(scan.take_exactly('.'));
            scan.take_while(|c| c.is_ascii_digit());
        }
        _ => (),
    }
    // TODO: 1234hello should probably be an error, not a number followed by an identifier.
    // But 1234+hello is ok.
    // TODO: Error if the f64 parse fails (but I don't think it ever can?)
    let val: f64 = scan.current_token().parse().unwrap();
    Tok::Number(val)
}

fn string(scan: &mut Scan) -> Result<Token, Error> {
    // TODO: Handle backslash escapes.
    let mut s = String::new();
    while let Some(c) = scan.take_if(|c| *c != '"') {
        s.push(c)
    }
    if !scan.take_exactly('"') {
        return Err(Error {
            place: scan.token_start(),
            kind: ErrorKind::UnterminatedString,
        });
    }
    Ok(Token {
        tok: Tok::String(s),
        place: scan.token_start(),
        lexeme: scan.current_token().to_owned(),
    })
}

fn word(scan: &mut Scan) -> Tok {
    scan.take_while(|c| c.is_ascii_alphanumeric() || *c == '_');
    match scan.current_token() {
        "and" => Tok::And,
        "class" => Tok::Class,
        "else" => Tok::Else,
        "false" => Tok::False,
        "for" => Tok::For,
        "fun" => Tok::Fun,
        "if" => Tok::If,
        "nil" => Tok::Nil,
        "or" => Tok::Or,
        "print" => Tok::Print,
        "return" => Tok::Return,
        "super" => Tok::Super,
        "this" => Tok::This,
        "true" => Tok::True,
        "var" => Tok::Var,
        "while" => Tok::While,
        s => Tok::Identifier(s.to_owned()),
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    fn lex_tokens(s: &str) -> Vec<Token> {
        let results = lex(s);
        results.into_iter().map(Result::unwrap).collect()
    }

    fn lex_toks<'s>(s: &'s str) -> Vec<Tok> {
        lex_tokens(s).into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn can_scan_integer() {
        assert_eq!(
            lex_tokens("12345"),
            &[Token {
                tok: Tok::Number(12345.0),
                place: Place::new(1, 1),
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
            lex_tokens("1\n// two would be here\n\n    3.000\n\n// the end!\n"),
            vec![
                Token {
                    tok: Tok::Number(1.0),
                    place: Place::new(1, 1),
                    lexeme: "1".to_owned(),
                },
                Token {
                    tok: Tok::Number(3.0),
                    place: Place::new(4, 5),
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
                place: Place::new(1, 1),
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
                place: Place::new(1, 1),
                lexeme: src.to_owned(),
            }]
        );
    }

    #[test]
    fn unterminated_string_error() {
        assert_eq!(
            lex("\"going along..."),
            [Err(Error {
                kind: ErrorKind::UnterminatedString,
                place: Place::file_start(),
            })]
        );
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

    #[test]
    fn column_positions_understand_tabs() {
        let tokens = lex_tokens(
            "
\tone_tab
\t\ttwo_tabs
\t\t    two_tabs_and_space
    \ttab_after_spaces
between\tthese\t\twords
    ",
        );
        assert_eq!(tokens.len(), 7);
        assert!(tokens
            .iter()
            .all(|token| matches!(token.tok, Tok::Identifier(_))));
        assert_eq!(
            tokens
                .iter()
                .map(|t| (t.place.line, t.place.column))
                .collect::<Vec<_>>(),
            &[(2, 9), (3, 17), (4, 21), (5, 9), (6, 1), (6, 9), (6, 25)]
        );
    }

    #[test]
    fn ignore_shebang() {
        let tokens = lex_tokens("#! mbplox --yolo\n#! maybe also a second line\n123\n");
        assert_eq!(
            tokens,
            [Token {
                tok: Tok::Number(123.0),
                place: Place::new(3, 1),
                lexeme: "123".to_owned(),
            }]
        );
    }

    #[test]
    fn lex_result_mixes_tokens_and_multiple_errors_in_order() {
        let unexpected_hash = ErrorKind::UnexpectedCharacter('#');
        assert_eq!(
            lex("hash##bang\n"),
            [
                Ok(Token {
                    tok: Tok::Identifier("hash".to_owned()),
                    place: Place::new(1, 1),
                    lexeme: "hash".to_owned(),
                }),
                Err(Error {
                    place: Place::new(1, 5),
                    kind: unexpected_hash.clone(),
                }),
                Err(Error {
                    place: Place::new(1, 6),
                    kind: unexpected_hash,
                }),
                Ok(Token {
                    tok: Tok::Identifier("bang".to_owned()),
                    place: Place::new(1, 7),
                    lexeme: "bang".to_owned(),
                }),
            ]
        );
    }
}
