use std::process::Command;

#[test]
fn project_architecture_rules_hold() {
    let status = Command::new("bash")
        .arg("../scripts/validation/check-architecture.sh")
        .status()
        .expect("architecture validation script should run");

    assert!(status.success());
}
