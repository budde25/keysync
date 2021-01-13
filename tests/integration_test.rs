use assert_cmd::Command;

#[test]
fn test_no_args() {
    let output_err =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap().unwrap_err();
    let output = output_err.as_output().unwrap();
    assert_eq!(output.status.code().unwrap(), 1);
}

#[test]
fn test_get_dry_run() {
    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("get")
        .arg("budde25")
        .arg("--dry-run")
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
#[ignore = "requires sudo"]
fn test_set_dry_run() {
    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("set")
        .arg("budde25")
        .arg("daily")
        .arg("--dry-run")
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
}
