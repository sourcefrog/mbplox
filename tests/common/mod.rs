// Copyright 2021 Martin Pool

//! Common helpers for integration tests.

use assert_cmd::Command;

/// Construct a [Command] that will run the `mbplox` interpreter when launched.
pub fn mbplox() -> Command {
    Command::cargo_bin("mbplox").unwrap()
}
