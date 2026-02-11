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

// ─── completions ───────────────────────────────────────────

#[test]
fn completions_bash() {
    portzap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("portzap"));
}

#[test]
fn completions_zsh() {
    portzap()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("portzap"));
}

#[test]
fn completions_fish() {
    portzap()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("portzap"));
}

#[test]
fn completions_invalid_shell() {
    portzap()
        .args(["completions", "nushell"])
        .assert()
        .failure();
}

// ─── free ──────────────────────────────────────────────────

#[test]
fn free_finds_unused_port() {
    // High port unlikely to be in use
    portzap()
        .args(["free", "59999"])
        .assert()
        .success()
        .stdout(predicate::str::contains("59999"));
}

#[test]
fn free_skips_occupied_port() {
    let guard = ListenerGuard::random();
    let port = guard.port().to_string();

    // The free port returned should NOT be the occupied port
    let output = portzap()
        .args(["free", &port])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let found_port: u16 = stdout.trim().parse().expect("should print a port number");
    assert!(found_port > guard.port(), "free should skip the occupied port");
}

#[test]
fn free_json_output() {
    portzap()
        .args(["free", "59999", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""port": 59999"#));
}

#[test]
fn free_no_port_in_range() {
    let guard = ListenerGuard::new(59998);
    let port = guard.port().to_string();

    // Search only the occupied port
    portzap()
        .args(["free", &port, "--max", &port])
        .assert()
        .failure();
}

// ─── wait ──────────────────────────────────────────────────

#[test]
fn wait_port_already_free() {
    // Port 59999 should be free — returns immediately
    portzap()
        .args(["wait", "59999", "--timeout", "2"])
        .assert()
        .success();
}

#[test]
fn wait_port_already_free_json() {
    portzap()
        .args(["wait", "59999", "--timeout", "2", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "free""#));
}

#[test]
fn wait_until_up_times_out() {
    // Port 59999 is free, waiting for it to become occupied should time out
    portzap()
        .args(["wait", "59999", "--until", "up", "--timeout", "1"])
        .assert()
        .failure();
}

#[test]
fn wait_until_up_times_out_json() {
    portzap()
        .args(["wait", "59999", "--until", "up", "--timeout", "1", "--format", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(r#""status": "timeout""#));
}

#[test]
fn wait_occupied_port_already_up() {
    let guard = ListenerGuard::random();
    let port = guard.port().to_string();

    portzap()
        .args(["wait", &port, "--until", "up", "--timeout", "2"])
        .assert()
        .success();
}
