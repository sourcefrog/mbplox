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
    While,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    /// 1-based source line where it occurs.
    pub line: usize,
    /// 1-based column for the start of the token.
    pub column: usize,
    /// Literal content of the lexeme.
    pub lexeme: String,
}

pub struct Error {
    // TODO
}

/// Lex some Lox source into a vec of tokens, and a vec of any errors encountered in scanning the
/// source.
pub fn lex(source: &str) -> (Vec<Token>, Vec<Error>) {
    let mut scan = Scan::new(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
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
            '"' => string(&mut scan),
            ch if ch.is_ascii_alphabetic() || ch == '_' => word(&mut scan),
            '#' if scan.next_column() == 2 && scan.take_exactly('!') => {
                // drop shebang line
                scan.take_until(|cc| *cc == '\n');
                continue;
            }
            other => panic!(
                "unhandled character {:?} on line {} column {}",
                other,
                scan.token_start_line(),
                scan.token_start_column(),
            ),
        };
        tokens.push(Token {
            tok,
            lexeme: scan.current_token().to_owned(),
            line: scan.token_start_line(),
            column: scan.token_start_column(),
        });
    }
    (tokens, errors)
}

fn number(scan: &mut Scan) -> Tok {
    scan.take_while(|c| c.is_ascii_digit());
    match scan.peek2() {
        Some(('.', cc)) if cc.is_ascii_digit() => {
            assert!(scan.take_exactly('.'));
            scan.take_while(|c| c.is_ascii_digit());
        }
        _ => (),
    }
    // TODO: 1234hello should probably be an error, not a number followed by an identifier.
    // But 1234+hello is ok.
    let val: f64 = scan.current_token().parse().unwrap();
    Tok::Number(val)
}

fn string(scan: &mut Scan) -> Tok {
    // TODO: Handle backslash escapes.
    // TODO: Clean error if the string is unterminated.
    let mut s = String::new();
    while let Some(c) = scan.take_if(|c| *c != '"') {
        s.push(c)
    }
    if !scan.take_exactly('"') {
        panic!(
            "unterminated string starting on line {} column {}",
            scan.token_start_line(),
            scan.token_start_column(),
        );
    }
    Tok::String(s)
}

fn word(scan: &mut Scan) -> Tok {
    scan.take_while(|c| c.is_ascii_alphanumeric() || *c == '_');
    match scan.current_token() {
        "true" => Tok::True,
        "false" => Tok::False,
        "nil" => Tok::Nil,
        s => Tok::Identifier(s.to_owned()),
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    fn lex_tokens(s: &str) -> Vec<Token> {
        let (tokens, errs) = lex(s);
        assert_eq!(errs.len(), 0);
        tokens
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
                line: 1,
                column: 1,
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
                    line: 1,
                    column: 1,
                    lexeme: "1".to_owned(),
                },
                Token {
                    tok: Tok::Number(3.0),
                    line: 4,
                    column: 5,
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
                column: 1,
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
                column: 1,
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
                .map(|t| (t.line, t.column))
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
                line: 3,
                column: 1,
                lexeme: "123".to_owned(),
            }]
        );
    }
}
