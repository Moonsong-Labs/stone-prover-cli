use std::path::Path;

use cairo_vm::air_private_input::{AirPrivateInput, AirPrivateInputSerializable};
use rstest::rstest;
use stone_prover_sdk::json::read_json_from_file;
use stone_prover_sdk::models::{Proof, Verifier};

use crate::common::cli_in_path;

mod common;

fn invoke_cli(
    with_bootloader: bool,
    executables: &[&Path],
    verifier: Option<Verifier>,
    prover_config: Option<&Path>,
    prover_parameters: Option<&Path>,
    output_file: Option<&Path>,
) -> Result<std::process::Output, std::io::Error> {
    let mut command = std::process::Command::new("stone-prover-cli");

    command.arg("prove");

    if with_bootloader {
        command.arg("--with-bootloader");
    }
    for executable in executables {
        command.arg(*executable);
    }

    if let Some(verifier) = verifier {
        command.arg("--verifier").arg(verifier.to_string());
    }
    if let Some(config_file) = prover_config {
        command.arg("--prover-config-file").arg(config_file);
    }
    if let Some(parameters_file) = prover_parameters {
        command.arg("--parameter-file").arg(parameters_file);
    }
    if let Some(output_file) = output_file {
        command.arg("--output-file").arg(output_file);
    }

    command.output()
}

fn assert_private_input_eq(
    private_input: AirPrivateInputSerializable,
    expected_private_input: AirPrivateInputSerializable,
) {
    fn remove_file_keys(private_input: &mut AirPrivateInput) {
        private_input.0.remove("trace_path");
        private_input.0.remove("memory_path");
    }

    let mut private_input = AirPrivateInput::from(private_input);
    let mut expected_private_input = AirPrivateInput::from(expected_private_input);

    remove_file_keys(&mut private_input);
    remove_file_keys(&mut expected_private_input);

    assert_eq!(private_input, expected_private_input);
}

fn assert_proof_eq(proof: Proof, expected_proof: Proof) {
    assert_private_input_eq(proof.private_input, expected_proof.private_input);
    assert_eq!(proof.public_input, expected_proof.public_input);
    assert_eq!(proof.prover_config, expected_proof.prover_config);
    assert_eq!(proof.proof_parameters, expected_proof.proof_parameters);
    assert_eq!(proof.proof_hex, expected_proof.proof_hex);
}

#[rstest]
fn execute_and_prove_program(
    #[from(cli_in_path)] _path: (),
    #[values(true, false)] provide_config: bool,
    #[values(true, false)] provide_parameters: bool,
) {
    let output_dir = tempfile::tempdir().unwrap();
    let proof_file = output_dir.path().join("proof.json");

    // Sanity check
    assert!(!proof_file.exists());

    let test_case_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("dependencies/cairo-programs/cairo0/fibonacci");

    let program = test_case_dir.join("fibonacci.json");
    let prover_config = test_case_dir.join("cpu_air_prover_config.json");
    let prover_parameters = test_case_dir.join("cpu_air_params.json");
    let expected_proof = test_case_dir.join("proof.json");

    let prover_config = match provide_config {
        true => Some(prover_config.as_path()),
        false => None,
    };

    let prover_parameters = match provide_parameters {
        true => Some(prover_parameters.as_path()),
        false => None,
    };

    let result = invoke_cli(
        false,
        &vec![program.as_path()],
        None,
        prover_config,
        prover_parameters,
        Some(proof_file.as_path()),
    )
    .expect("Command should succeed");

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8(result.stderr).unwrap()
    );

    assert!(proof_file.exists());

    let proof: Proof = read_json_from_file(proof_file).unwrap();
    let expected_proof: Proof = read_json_from_file(expected_proof).unwrap();
    assert_proof_eq(proof, expected_proof);
}

#[rstest]
fn execute_and_prove_program_l1_verifier(#[from(cli_in_path)] _path: ()) {
    let output_dir = tempfile::tempdir().unwrap();
    let proof_file = output_dir.path().join("proof.json");

    // Sanity check
    assert!(!proof_file.exists());

    let test_case_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("dependencies/cairo-programs/cairo0/fibonacci");

    let program = test_case_dir.join("fibonacci.json");

    let result = invoke_cli(
        false,
        &vec![program.as_path()],
        Some(Verifier::L1),
        None,
        None,
        Some(proof_file.as_path()),
    )
    .expect("Command should succeed");

    println!(
        "stdout: {}\n\n\nstderr: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );

    assert!(proof_file.exists());

    let proof: Proof = read_json_from_file(proof_file).unwrap();
    // TODO: test with L1 verifier
    // Check that the FRI steps are compatible with the L1 verifier
    assert_eq!(
        proof.proof_parameters.stark.fri.fri_step_list,
        vec![0, 2, 2, 2, 2, 2, 2, 2]
    );
    assert_eq!(proof.proof_parameters.stark.fri.last_layer_degree_bound, 32);
}

#[rstest]
fn execute_and_prove_program_with_bootloader(#[from(cli_in_path)] _path: ()) {
    let output_dir = tempfile::tempdir().unwrap();
    let proof_file = output_dir.path().join("proof.json");

    // Sanity check
    assert!(!proof_file.exists());

    let test_case_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("dependencies/cairo-programs/bootloader/programs/fibonacci");

    let program = test_case_dir.join("program.json");

    let result = invoke_cli(
        true,
        &vec![program.as_path()],
        None,
        None,
        None,
        Some(proof_file.as_path()),
    )
    .expect("Command should succeed");

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8(result.stderr).unwrap()
    );

    assert!(proof_file.exists());
}

#[rstest]
fn execute_and_prove_pie_with_bootloader(#[from(cli_in_path)] _path: ()) {
    let output_dir = tempfile::tempdir().unwrap();
    let proof_file = output_dir.path().join("proof.json");

    // Sanity check
    assert!(!proof_file.exists());

    let test_case_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("dependencies/cairo-programs/bootloader/pies/fibonacci-stone-e2e");

    let pie = test_case_dir.join("cairo_pie.zip");
    let expected_proof = test_case_dir.join("output/proof.json");

    let result = invoke_cli(
        true,
        &vec![pie.as_path()],
        None,
        None,
        None,
        Some(proof_file.as_path()),
    )
    .expect("Command should succeed");

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8(result.stderr).unwrap()
    );

    assert!(proof_file.exists());
    let proof: Proof = read_json_from_file(proof_file).unwrap();
    let expected_proof: Proof = read_json_from_file(expected_proof).unwrap();
    assert_proof_eq(proof, expected_proof);
}
