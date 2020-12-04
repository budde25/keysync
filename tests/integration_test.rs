use assert_cmd::Command;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd.unwrap();
    assert_eq!(output.status.code().unwrap(), 1);
}
