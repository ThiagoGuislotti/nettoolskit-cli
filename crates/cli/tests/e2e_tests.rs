//! End-to-end tests for the `ntk` binary
//!
//! Uses `assert_cmd` to exercise the compiled binary as a subprocess,
//! verifying exit codes, stdout/stderr output, and overall behavior.
//!
//! Manifest subcommands are supported non-interactively via clap
//! (`list`, `check`, `render`, `apply`) while preserving interactive mode
//! when no manifest subcommand is provided.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to build a `Command` pointing at the `ntk` binary.
fn ntk() -> Command {
    cargo_bin_cmd!("ntk")
}

fn valid_manifest_yaml() -> &'static str {
    r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test-manifest
solution:
  root: ./
  slnFile: TestSolution.sln
conventions:
  namespaceRoot: Acme.Test
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: artifact
  artifact:
    kind: value-object
contexts:
  - name: Sales
    aggregates: []
    useCases: []
templates:
  mapping:
    - artifact: value-object
      template: value-object.hbs
      dst: "{context}/ValueObjects/{name}.cs"
render:
  rules:
    - expand: "{{Context}}"
      as: ctx
guards:
  requireExistingProjects: false
"#
}

fn write_fixture(dir: &Path, relative: &str, content: &str) -> PathBuf {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create fixture directories");
    }
    fs::write(&path, content).expect("failed to write fixture file");
    path
}

fn create_manifest_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let manifest_path = write_fixture(dir.path(), "demo.manifest.yaml", valid_manifest_yaml());
    write_fixture(
        dir.path(),
        "templates/value-object.hbs",
        "namespace {{namespaceRoot}};\npublic class {{Name}} {}\n",
    );
    (dir, manifest_path)
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

#[test]
fn service_help_subcommand_returns_zero() {
    ntk()
        .args(["service", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("service"))
        .stdout(predicate::str::contains("--port"));
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

// ─── 4.2.3 / 4.2.4 / 4.2.5 — manifest sub-subcommands ─────────────────

#[test]
fn manifest_list_subcommand_returns_zero() {
    let (dir, _manifest_path) = create_manifest_fixture();
    ntk()
        .current_dir(dir.path())
        .args(["manifest", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 1 manifest file(s)"));
}

#[test]
fn manifest_check_subcommand_returns_zero() {
    let (dir, manifest_path) = create_manifest_fixture();
    let manifest = manifest_path.to_string_lossy().to_string();
    ntk()
        .current_dir(dir.path())
        .args(["manifest", "check", &manifest])
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn manifest_check_invalid_file_returns_error() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let invalid_manifest = write_fixture(
        dir.path(),
        "invalid.manifest.yaml",
        "apiVersion: ntk/v1\nkind: solution\nmeta:\n name: bad\nbad: [\n",
    );
    let manifest = invalid_manifest.to_string_lossy().to_string();

    ntk()
        .current_dir(dir.path())
        .args(["manifest", "check", &manifest])
        .assert()
        .failure()
        .stdout(predicate::str::contains("validation errors"));
}

#[test]
fn manifest_render_dry_run_subcommand_returns_zero() {
    let (dir, manifest_path) = create_manifest_fixture();
    let manifest = manifest_path.to_string_lossy().to_string();
    ntk()
        .current_dir(dir.path())
        .args(["manifest", "render", &manifest, "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Render preview completed"));
}
