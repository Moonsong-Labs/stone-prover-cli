use crate::common::cli_in_path;
use rstest::rstest;
use std::path::Path;

mod common;

// "0x3106b7628a3cbddadb733ea96284977cc21e890d61aa6dee00badcbd90065ce"

fn invoke_cli(program_file: &Path) -> Result<std::process::Output, std::io::Error> {
    let mut command = std::process::Command::new("stone-prover-cli");
    command.arg("hash").arg(program_file);

    command.output()
}

#[rstest]
fn test_hash_bootloader(#[from(cli_in_path)] _path: ()) {
    let expected_hash = "0x3106b7628a3cbddadb733ea96284977cc21e890d61aa6dee00badcbd90065ce";

    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("dependencies/cairo-programs/bootloader/bootloader-v0.13.0.json");

    let output = invoke_cli(program_file.as_path()).expect("Command should succeed");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let hash = String::from_utf8_lossy(&output.stdout);

    assert_eq!(hash.trim(), expected_hash);
}
