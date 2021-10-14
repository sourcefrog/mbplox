// Copyright 2021 Martin Pool

//! Read sample files and check that the results are as given in comments.

mod common;
mod expect;

use std::path::Path;

use expect::assert_expected_for_dir;

/// For every source file in the `lex` subdirectory, the tokens recognized by
/// the lexer match those given in the `// expect:` comments within the file.
#[test]
fn lex_tokens_from_testdata() {
    assert_expected_for_dir(&Path::new("testdata/lex"), &["--dump-tokens"]);
}
