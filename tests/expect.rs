// Copyright 2021 Martin Pool

//! Check the Lox programs in `testdata` against their included expectations.
//!
//! Aside from Lox source code, testdata filer can include:
//!
//! * `// expect: ` comments, whose text should be produced on stdout.
//!
//! * `#! ` shebang lines, supplying args for the interpreter. The first word should be `mbplox`,
//! but the interpreter is actually found in the Cargo build directory.

#![cfg(test)]

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;

/// For every source file in the `testdata` directory, run the interpreter with optionr given in
/// the file, and check the output matches the expectations.
#[test]
fn testdata_cases() {
    let mut found_any_files = false;
    for file_path in walkdir::WalkDir::new("testdata")
        .into_iter()
        .map(Result::unwrap)
        .map(|de| de.into_path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("lox"))
    {
        found_any_files = true;
        let case = Case::from_file(&file_path);
        case.assert();
    }
    assert!(found_any_files);
}

/// A description of how to run a test ans what to expect, extracted from a testdata file.
#[derive(Debug, Clone, PartialEq)]
struct Case {
    /// Arguments to the interpreter, not including the filename.
    args: Vec<String>,
    /// Lox test file name.
    path: PathBuf,
    /// Expected output lines.
    output: Vec<String>,
}

impl Case {
    /// Construct a case by extracting annotations from a file.
    pub fn from_file(path: &Path) -> Case {
        let path = path.to_owned();
        let source = fs::read_to_string(&path).unwrap();

        let mut output = Vec::new();
        for l in source.lines() {
            if let Some((_, expectation)) = l.split_once("// expect: ") {
                output.push(expectation.to_owned())
            }
        }

        let mut args = Vec::new();
        if let Some(shebang) = source.lines().next().and_then(|l| l.strip_prefix("#!")) {
            let mut words = shebang.split_ascii_whitespace();
            assert_eq!(words.next(), Some("mbplox"));
            args = words.map(|w| w.to_owned()).collect();
        }

        Case { path, output, args }
    }

    /// Run mbplox on a file with given arguments, and check that the output matches the expectations
    /// in the file.
    fn assert(&self) {
        println!("mbplox {} {}", self.args.join(" "), self.path.display());
        let output = common::mbplox()
            .args(&self.args)
            .arg(&self.path)
            .output()
            .unwrap();
        if !output.stderr.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }
        assert!(output.status.success());
        // Possibly this should compare the multi-line strings, rather than lists of strings, but that
        // would need more care to work consistently on Windows...
        let output_string = String::from_utf8(output.stdout).unwrap(); // hold the str
        let output_lines: Vec<&str> = output_string.lines().collect();
        assert_eq!(output_lines, self.output);
    }
}
