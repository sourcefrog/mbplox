// Copyright 2021 Martin Pool

//! Scan text as characters, with peek-ahead, tracking line numbers,
//! and remembering the characters in each token.

/// Scan characters with arbitrary lookahead.
///
/// Provides low-level char parsing without knowing anything specific about the
/// grammar.
///
/// Type parameter `C` is typically `char` but could be `u8` etc.
pub struct Scan<C, I>
where
    I: Iterator<Item = C>,
    C: PartialEq + Clone + IsNewline,
{
    inner: I,
    buf: Vec<C>,
    current_token: Vec<C>,
    token_start_line: usize,
    line_number: usize,
}

impl<C, I> Scan<C, I>
where
    I: Iterator<Item = C>,
    C: PartialEq + Clone + IsNewline,
{
    pub fn new(inner: I) -> Scan<C, I> {
        Scan {
            inner,
            buf: Vec::new(),
            current_token: Vec::new(),
            line_number: 1,
            token_start_line: 1,
        }
    }

    pub fn start_token(&mut self) {
        self.current_token.clear();
        self.token_start_line = self.line_number;
    }

    /// Return all the atoms recognized since the last [Scan::start_token].
    pub fn current_token<S>(&self) -> S
    where
        S: std::iter::FromIterator<C>,
    {
        self.current_token.iter().cloned().collect::<S>()
    }

    pub fn current_token_start_line(&self) -> usize {
        self.token_start_line
    }

    /// Consume and return one character.
    ///
    /// All consumption should go through here to maintain invariants, including
    /// line numbering and accumulating the current token.
    pub fn take(&mut self) -> Option<C> {
        let c = if self.buf.is_empty() {
            self.inner.next()?
        } else {
            self.buf.remove(0)
        };
        if c.is_newline() {
            self.line_number += 1;
        }
        self.current_token.push(c.clone());
        Some(c)
    }

    pub fn take_if<F>(&mut self, f: F) -> Option<C>
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

    pub fn take_exactly(&mut self, c: &C) -> Option<C> {
        self.take_if(|cc| *cc == *c)
    }

    pub fn peek(&mut self) -> Option<&C> {
        self.peek_nth(0)
    }

    pub fn peek2(&mut self) -> Option<(&C, &C)> {
        if self.peek_nth(1).is_some() {
            Some((&self.buf[0], &self.buf[1]))
        } else {
            None
        }
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<&C> {
        while self.buf.len() <= n {
            if let Some(c) = self.inner.next() {
                self.buf.push(c)
            } else {
                return None;
            }
        }
        Some(&self.buf[n])
    }

    pub fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }
}

pub trait IsNewline {
    fn is_newline(&self) -> bool;
}

impl IsNewline for char {
    fn is_newline(&self) -> bool {
        *self == '\n'
    }
}
