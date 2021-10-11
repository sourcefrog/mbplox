// Copyright 2021 Martin Pool

//! An interpreter for the Lox small language from *Crafting Interpreters*.

// Just while half-implemented.
#![allow(unused, dead_code, unused_imports)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

mod ast;
mod eval;
mod lex;
mod parse;
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
    if args.file.is_none() && args.eval.is_empty() {
        eprintln!(
            "error: repl is not implemented yet: suppply either a source file name or --eval arguments"
        );
        std::process::exit(sysexit::Code::Usage as i32);
    }
    if let Some(path) = &args.file {
        all_sources.push(fs::read_to_string(path).context("read source file")?);
    }
    all_sources.extend(args.eval);
    // TODO: If no sources then repl.
    if args.dump_tokens {
        for source in &all_sources {
            let (tokens, _errs) = lex::lex(source);
            for token in tokens {
                println!("{:?}", token.tok);
            }
        }
    } else {
        let mut interpreter = eval::Interpreter::new();
        for source in &all_sources {
            let value = interpreter.eval(source)?;
            println!("{}", value);
        }
    }
    Ok(())
}
