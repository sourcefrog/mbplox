// Copyright 2021 Martin Pool

use std::fs;
// use std::io;
use std::path::Path;

use anyhow::{Context, Result};

mod lex;
mod scan;

use lex::Token;

fn main() -> Result<()> {
    let mut args = std::env::args();
    run_file(Path::new(&args.nth(1).expect("one argument")))
}

fn run_file(path: &Path) -> Result<()> {
    run(&fs::read_to_string(path).context("read source file")?)
}

fn run(source: &str) -> Result<()> {
    let tokens: Vec<Token> = lex::lex(source).collect();
    dbg!(tokens);
    Ok(())
}
