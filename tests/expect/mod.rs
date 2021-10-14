// Copyright 2021 Martin Pool

//! Handle the `// expect: ` markers that show expected output within Lox
//! test source files.

#![cfg(test)]

use std::fs;
use std::path::Path;

use pretty_assertions::assert_eq;

use crate::common;

/// Run mbplox on every `.lox` file in a given directory, and check the output
/// matches the given expectations.
pub fn assert_expected_for_dir(dir: &Path, args: &[&str]) {
    let mut found_any_files = false;
    for file_path in fs::read_dir(dir)
        .unwrap()
        .map(Result::unwrap)
        .map(|de| de.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("lox"))
    {
        found_any_files = true;
        assert_expected(&file_path, args);
    }
    assert!(found_any_files);
}

/// Run mbplox on a file with given arguments, and check that the output matches the expectations
/// in the file.
fn assert_expected(filename: &Path, args: &[&str]) {
    println!("mbplox {} {}", args.join(" "), filename.display());
    let output = common::mbplox().args(args).arg(filename).output().unwrap();
    if !output.stderr.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
    // Possibly this should compare the multi-line strings, rather than lists of strings, but that
    // would need more care to work consistently on Windows...
    let output_string = String::from_utf8(output.stdout).unwrap();
    let output_lines: Vec<&str> = output_string.lines().collect();
    assert_eq!(output_lines, expectations_from_file(filename));
}

/// Return the contents of the `// expect: ` comments in a given path, relative to the `tests`
/// directory.
fn expectations_from_file<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    extract_expectations(&fs::read_to_string(path.as_ref()).unwrap())
        .into_iter()
        .map(ToOwned::to_owned)
        .collect()
}

/// Find all "expect: " comments in a source string, and return the contents
/// of the expectations (without the markers.)
fn extract_expectations(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let pattern: &str = "// expect: ";
    for l in s.lines() {
        if let Some((_, expectation)) = l.split_once(pattern) {
            result.push(expectation)
        }
    }
    result
}
