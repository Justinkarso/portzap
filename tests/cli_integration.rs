mod helpers;

use assert_cmd::Command;
use helpers::ListenerGuard;
use predicates::prelude::*;

fn portzap() -> Command {
    Command::cargo_bin("portzap").unwrap()
}

#[test]
fn no_args_shows_help() {
    portzap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn version_flag() {
    portzap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("portzap"));
}

#[test]
fn invalid_port_text() {
    portzap()
        .arg("abc")
        .assert()
        .failure();
}

#[test]
fn invalid_port_too_large() {
    portzap()
        .arg("99999")
        .assert()
        .failure();
}

#[test]
fn no_process_on_port() {
    // Use a high port unlikely to be in use
    portzap()
        .args(["list", "59999"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No processes found"));
}

#[test]
fn list_finds_listening_process() {
    let guard = ListenerGuard::random();
    let port = guard.port().to_string();

    portzap()
        .args(["list", "--format", "json", &port])
        .assert()
        .success()
        .stdout(predicate::str::contains(&format!("\"port\": {}", guard.port())));
}

#[test]
fn dry_run_does_not_kill() {
    let guard = ListenerGuard::random();
    let port = guard.port().to_string();

    portzap()
        .args(["--dry-run", &port])
        .assert()
        .success()
        .stderr(predicate::str::contains("dry-run"));

    // Process should still be alive — verify by listing again
    portzap()
        .args(["list", "--format", "json", &port])
        .assert()
        .success()
        .stdout(predicate::str::contains(&format!("\"port\": {}", guard.port())));
}

#[test]
fn json_output_is_valid() {
    let guard = ListenerGuard::random();
    let port = guard.port().to_string();

    let output = portzap()
        .args(["list", "--format", "json", &port])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());

    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["port"], guard.port());
}

#[test]
fn plain_output_format() {
    let guard = ListenerGuard::random();
    let port = guard.port().to_string();

    portzap()
        .args(["list", "--format", "plain", &port])
        .assert()
        .success()
        .stdout(predicate::str::contains(&format!("\t{}\t", guard.port())));
}

#[test]
fn port_range_syntax() {
    // Range where nothing is listening
    portzap()
        .args(["--dry-run", "59990-59995"])
        .assert()
        .success();
}

#[test]
fn list_all_listening() {
    // Just verify it doesn't crash — output depends on system state
    portzap()
        .arg("list")
        .assert()
        .success();
}

#[test]
fn kill_subcommand_works() {
    portzap()
        .args(["kill", "--dry-run", "59999"])
        .assert()
        .success();
}
