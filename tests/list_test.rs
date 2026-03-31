use assert_cmd::Command;

#[test]
fn test_list_json_outputs_valid_json() {
    let mut cmd = Command::cargo_bin("chronos").unwrap();
    let output = cmd.arg("list").arg("--json").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Expected valid JSON, got: {stdout}");
}

#[test]
fn test_list_table_has_headers() {
    let mut cmd = Command::cargo_bin("chronos").unwrap();
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicates::str::contains("ID"));
}
