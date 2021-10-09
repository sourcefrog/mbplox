// Copyright 2021 Martin Pool

//! Read sample files and check that the results are as given in comments.

use std::fs;

/// Find "expect: " comments and return everything after them.
fn extract_expectations(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let pattern : &str = "// expect: ";
    for l in s.lines() {
        if let Some((_, expectation)) = l.split_once(pattern) {
            result.push(expectation)
        }
    }
    result
}

#[test] 
fn extract_expectations_from_literals() {
    let filename = "samples/literals.lox";
    let source = fs::read_to_string(filename).unwrap();
    let expectations = extract_expectations(&source);
    assert_eq!(expectations, vec!["1234"]);
}
