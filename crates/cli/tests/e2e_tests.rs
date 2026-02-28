//! End-to-end tests for the `ntk` binary
//!
//! Uses `assert_cmd` to exercise the compiled binary as a subprocess,
//! verifying exit codes, stdout/stderr output, and overall behavior.
//!
//! NOTE: `ntk manifest` sub-subcommands (check, apply, render, list) are
//! currently interactive-only (via `inquire` menus). Non-interactive E2E
//! tests for those commands require refactoring `Commands::Manifest` to
//! accept clap sub-subcommands. See Phase 4.2.3–4.2.5 in the tracker.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to build a `Command` pointing at the `ntk` binary.
fn ntk() -> Command {
    cargo_bin_cmd!("ntk")
}

// ─── 4.2.2 — ntk --help ─────────────────────────────────────────────────

#[test]
fn help_flag_returns_zero_and_shows_usage() {
    ntk()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("ntk"));
}

#[test]
fn version_flag_returns_zero() {
    ntk()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0"));
}

#[test]
fn help_shows_manifest_subcommand() {
    ntk()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("manifest"));
}

#[test]
fn help_shows_translate_subcommand() {
    ntk()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("translate"));
}

// ─── translate subcommand ────────────────────────────────────────────────

#[test]
fn translate_without_args_fails() {
    ntk()
        .arg("translate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn translate_missing_path_fails() {
    ntk()
        .args(["translate", "--from", "csharp", "--to", "rust"])
        .assert()
        .failure();
}

// ─── 4.2.6 — unknown subcommand / bad args ──────────────────────────────

#[test]
fn unknown_subcommand_fails() {
    ntk()
        .arg("nonexistent-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn verbose_flag_is_accepted() {
    // --verbose with --help should succeed (verbose is a global flag)
    ntk().args(["--verbose", "--help"]).assert().success();
}
