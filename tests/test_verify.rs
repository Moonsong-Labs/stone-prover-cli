use std::path::Path;

use rstest::rstest;

use crate::common::cli_in_path;

mod common;

fn invoke_cli(proof_file: &Path) -> Result<std::process::Output, std::io::Error> {
    let mut command = std::process::Command::new("stone-prover-cli");
    command.arg("verify").arg(proof_file);

    command.output()
}

#[rstest]
fn test_verify_program(#[from(cli_in_path)] _path: ()) {
    let test_case_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("dependencies/cairo-programs/cairo0/fibonacci");
    let proof_file = test_case_dir.join("proof.json");

    invoke_cli(proof_file.as_path()).expect("Command should succeed");
}

#[rstest]
fn test_verify_pie_with_bootloader(#[from(cli_in_path)] _path: ()) {
    let test_case_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("dependencies/cairo-programs/bootloader/fibonacci-stone-e2e");
    let proof_file = test_case_dir.join("output/proof.json");

    invoke_cli(proof_file.as_path()).expect("Command should succeed");
}
