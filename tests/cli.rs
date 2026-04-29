use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE___goto_bin")
}

#[test]
fn add_and_resolve_alias() {
    let target_dir = tempdir().expect("target tempdir should be created");
    let config_dir = tempdir().expect("config tempdir should be created");
    let expected = target_dir
        .path()
        .canonicalize()
        .expect("target tempdir should canonicalize");

    Command::new(bin())
        .args([
            "add",
            "myalias",
            target_dir.path().to_str().expect("utf-8 path"),
        ])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .assert()
        .success();

    let output = Command::new(bin())
        .args(["resolve", "myalias"])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .output()
        .expect("resolve command should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be utf-8"),
        expected.to_string_lossy()
    );
}

#[test]
fn no_args_lists_aliases() {
    let target_dir = tempdir().expect("target tempdir should be created");
    let config_dir = tempdir().expect("config tempdir should be created");

    Command::new(bin())
        .args([
            "add",
            "proj",
            target_dir.path().to_str().expect("utf-8 path"),
        ])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .assert()
        .success();

    let output = Command::new(bin())
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .output()
        .expect("list command should run");

    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .expect("stdout should be utf-8")
        .contains("proj"));
}

#[test]
fn resolve_unknown_alias_exits_one() {
    let config_dir = tempdir().expect("config tempdir should be created");

    Command::new(bin())
        .args(["resolve", "does-not-exist"])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn remove_alias() {
    let target_dir = tempdir().expect("target tempdir should be created");
    let config_dir = tempdir().expect("config tempdir should be created");

    Command::new(bin())
        .args([
            "add",
            "todelete",
            target_dir.path().to_str().expect("utf-8 path"),
        ])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .assert()
        .success();

    Command::new(bin())
        .args(["remove", "todelete"])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .assert()
        .success();

    Command::new(bin())
        .args(["resolve", "todelete"])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn rejects_nonexistent_directory() {
    let config_dir = tempdir().expect("config tempdir should be created");

    Command::new(bin())
        .args(["add", "bad", "/this/definitely/does/not/exist"])
        .env("GOTO_CONFIG_DIR", config_dir.path())
        .assert()
        .failure()
        .code(1);
}
