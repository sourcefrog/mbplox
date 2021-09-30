// Copyright 2021 Martin Pool

use std::fs;
// use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

mod lex;
mod scan;

use argh::FromArgs;

#[derive(FromArgs)]
/// Run a Lox program.
struct Args {
    /// file to interpret
    #[argh(positional)]
    file: Option<PathBuf>,

    /// lox source code to run
    #[argh(option, short = 'e')]
    eval: Vec<String>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    if let Some(path) = &args.file {
        run_file(path)?
    }
    for expr in args.eval {
        run(&expr)?
    }
    Ok(())
}

fn run_file(path: &Path) -> Result<()> {
    run(&fs::read_to_string(path).context("read source file")?)
}

fn run(source: &str) -> Result<()> {
    let lexer = lex::Lexer::new(source);
    dbg!(lexer.tokens());
    Ok(())
}
