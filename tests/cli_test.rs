use assert_cmd::Command;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("chronos").unwrap();
    cmd.arg("--help").assert().success().stdout(predicates::str::contains("chronos"));
}

#[test]
fn test_no_args_shows_help() {
    let mut cmd = Command::cargo_bin("chronos").unwrap();
    cmd.assert().failure();
}
