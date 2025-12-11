use assert_cmd::Command;
use predicates::prelude::*;

fn cc_switch() -> Command {
    Command::cargo_bin("cc-switch").unwrap()
}

#[test]
fn test_help() {
    cc_switch()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage AI provider configurations"));
}

#[test]
fn test_version() {
    cc_switch()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cc-switch"));
}

#[test]
fn test_provider_help() {
    cc_switch()
        .args(["provider", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Provider management"));
}

#[test]
fn test_provider_list() {
    cc_switch()
        .args(["provider", "list"])
        .assert()
        .success();
}

#[test]
fn test_provider_list_json() {
    cc_switch()
        .args(["provider", "list", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn test_provider_list_invalid_app() {
    cc_switch()
        .args(["provider", "list", "--app", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid app type"));
}

#[test]
fn test_provider_show_nonexistent() {
    cc_switch()
        .args(["provider", "show", "nonexistent-id-12345"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provider not found"));
}

#[test]
fn test_provider_switch_nonexistent() {
    cc_switch()
        .args(["provider", "switch", "nonexistent-id-12345"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provider not found"));
}

#[test]
fn test_provider_delete_nonexistent() {
    cc_switch()
        .args(["provider", "delete", "nonexistent-id-12345", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provider not found"));
}

#[test]
fn test_provider_add_missing_name() {
    cc_switch()
        .args(["provider", "add", "--api-key", "test"])
        .write_stdin("\n") // Empty name
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be empty"));
}

#[test]
fn test_provider_export() {
    cc_switch()
        .args(["provider", "export"])
        .assert()
        .success();
}
