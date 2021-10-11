// Copyright 2021 Martin Pool

//! Common helpers for integration tests.

use assert_cmd::Command;

pub fn main_command() -> Command {
    Command::cargo_bin("mbplox").unwrap()
}
