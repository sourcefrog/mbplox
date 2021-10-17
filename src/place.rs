// Copyright 2021 Martin Pool

//! A (line, column) location in the source, for error reporting.

use std::fmt;

/// A (line, column) location in the source, for error reporting.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Place {
    /// 1-based line number.
    pub line: usize,

    /// 1-based column.
    ///
    /// Measured in chars.
    pub column: usize,
}

impl Place {
    /// Construct a new Place (1,1), the start of a file.
    pub fn file_start() -> Place {
        Place::new(1, 1)
    }

    /// Construct a new Place at the given line, column.
    pub fn new(line: usize, column: usize) -> Place {
        assert!(line >= 1);
        assert!(column >= 1);
        Place { line, column }
    }

    /// Advance by one character, accounting for tabs and newlines.
    pub fn advance(&mut self, c: char) {
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else if c == '\t' {
            self.column += 1;
            while self.column % 8 != 1 {
                self.column += 1;
            }
        } else {
            self.column += 1;
        }
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {} column {}", self.line, self.column)
    }
}
