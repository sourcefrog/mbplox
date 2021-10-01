// Copyright 2021 Martin Pool

//! An interpreter for the Lox small language from *Crafting Interpreters*.

use std::fs;
// use std::io;
use std::path::PathBuf;

use anyhow::{Context, Result};

mod ast;
mod eval;
mod lex;
mod scan;
mod value;

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

    /// print all the tokens from the input, instead of running it.
    #[argh(switch)]
    dump_tokens: bool,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let mut all_sources: Vec<String> = Vec::new();
    if let Some(path) = &args.file {
        all_sources.push(fs::read_to_string(path).context("read source file")?);
    }
    all_sources.extend(args.eval);
    if args.dump_tokens {
        for source in all_sources {
            let lexer = lex::Lexer::new(&source);
            for token in lexer.tokens() {
                println!("{:?}", token.tok);
            }
        }
    } else {
        unimplemented!()
    }
    Ok(())
}
