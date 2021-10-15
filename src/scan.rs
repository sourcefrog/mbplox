// Copyright 2021 Martin Pool

//! Scan text as characters, with peek-ahead, tracking line numbers,
//! and remembering the characters in each token.
//!
//! This layer knows nothing about the syntax of Lox, only how to generically scan a text file.

/// Scan characters with arbitrary lookahead.
///
/// Provides low-level char parsing without knowing anything specific about the
/// grammar.
pub struct Scan<'a> {
    input: std::str::Chars<'a>,
    lookahead: Vec<char>,
    current_token: String,
    token_start_line: usize,
    line_number: usize,
    /// 1-based column of the character *about to be* taken.
    column: usize,
    /// 1-based column of the first character of the current token.
    token_start_column: usize,
}

impl<'a> Scan<'a> {
    pub fn new(source: &'a str) -> Scan<'a> {
        Scan {
            input: source.chars(),
            lookahead: Vec::new(),
            current_token: String::new(),
            line_number: 1,
            column: 1,
            token_start_line: 1,
            token_start_column: 1,
        }
    }

    pub fn start_token(&mut self) {
        self.current_token.clear();
        self.token_start_line = self.line_number;
        self.token_start_column = self.column;
    }

    /// Return all the atoms recognized since the last [Scan::start_token].
    pub fn current_token(&self) -> &str {
        &self.current_token
    }

    pub fn token_start_line(&self) -> usize {
        self.token_start_line
    }

    /// Return the 1-based column at which the current token started.
    pub fn token_start_column(&self) -> usize {
        self.token_start_column
    }

    /// Return the 1-based column of the next character that will be returned by [Scan::take].
    pub fn next_column(&self) -> usize {
        self.column
    }

    /// Consume and return one character.
    ///
    /// All consumption should go through here to maintain invariants, including
    /// line numbering and accumulating the current token.
    ///
    /// Returns None at the end of the input.
    pub fn take(&mut self) -> Option<char> {
        let c = if self.lookahead.is_empty() {
            self.input.next()?
        } else {
            self.lookahead.remove(0)
        };
        if c == '\n' {
            self.line_number += 1;
            self.column = 1;
        } else if c == '\t' {
            // Increment at least one column, and continue until reaching an 8-aligned tab stop.
            loop {
                self.column += 1;
                if self.column % 8 == 1 {
                    break;
                }
            }
        } else {
            self.column += 1;
        }
        self.current_token.push(c.clone());
        Some(c)
    }

    /// Consume and return the next character if it matches a predicate,
    /// otherwise leave it alone and return None.
    ///
    /// Returns None at the end of the input.
    pub fn take_if<F>(&mut self, f: F) -> Option<char>
    where
        F: Fn(&char) -> bool,
    {
        self.peek().filter(|c| f(c)).and_then(|_c| self.take())
    }

    /// Consume characters while they match a predicate.
    ///
    /// Consumed characters are accumulated into current_token but not returned.
    pub fn take_while<F>(&mut self, f: F)
    where
        F: Fn(&char) -> bool,
    {
        while self.take_if(&f).is_some() {}
    }

    /// Take characters up to and including a terminator.
    ///
    /// Consumed characters are accumulated into current_token but not returned.
    pub fn take_until(&mut self, f: fn(&char) -> bool) {
        while let Some(c) = self.take() {
            if f(&c) {
                break;
            }
        }
    }

    /// If the next character is `c` then consume it and return true;
    /// otherwise leave it alone and return false.
    pub fn take_exactly(&mut self, c: char) -> bool {
        self.take_if(|cc| *cc == c).is_some()
    }

    /// Peek at the next character, if there is one, without consuming it.
    pub fn peek(&mut self) -> Option<char> {
        self.peek_nth(0)
    }

    /// Peek at the next two characters, if there are two more characters, without consuming them.
    pub fn peek2(&mut self) -> Option<(char, char)> {
        if self.peek_nth(1).is_some() {
            Some((self.lookahead[0], self.lookahead[1]))
        } else {
            None
        }
    }

    fn peek_nth(&mut self, n: usize) -> Option<char> {
        while self.lookahead.len() <= n {
            if let Some(c) = self.input.next() {
                self.lookahead.push(c)
            } else {
                return None;
            }
        }
        Some(self.lookahead[n])
    }

    /// Return true if the scanner is at the end of the input.
    pub fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }
}
