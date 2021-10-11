// Copyright 2021 Martin Pool

//! Tests for the command-line interface itself: arguments etc.

mod common;
use common::main_command;

#[test]
fn error_if_no_args() {
    // TODO: Later, this should start a repl instead of erroring.
    main_command().assert().failure();
}
