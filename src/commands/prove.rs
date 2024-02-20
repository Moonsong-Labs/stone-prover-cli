use std::fs::File;
use std::path::{Path, PathBuf};

use cairo_vm::hint_processor::builtin_hint_processor::bootloader::types::{Task, TaskSpec};
use cairo_vm::types::errors::cairo_pie_error::CairoPieError;
use cairo_vm::types::errors::program_errors::ProgramError;
use cairo_vm::types::program::Program;
use cairo_vm::vm::runners::cairo_pie::CairoPie;
use serde::Serialize;
use stone_prover_sdk::cairo_vm::{
    extract_execution_artifacts, run_bootloader_in_proof_mode, run_in_proof_mode,
    ExecutionArtifacts, ExecutionError,
};
use stone_prover_sdk::error::ProverError;
use stone_prover_sdk::fri::generate_prover_parameters;
use stone_prover_sdk::models::{Layout, ProverConfig};
use stone_prover_sdk::prover::run_prover;

use crate::cli::{ProveBareArgs, ProveCommand, ProveWithBootloaderArgs};
use crate::toolkit::json::{read_json_from_file, ReadJsonError};

const BOOTLOADER_PROGRAM: &[u8] =
    include_bytes!("../../dependencies/cairo-programs/bootloader/bootloader-v0.13.0.json");

fn write_json_to_file<T: Serialize, P: AsRef<Path>>(obj: T, path: P) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    serde_json::to_writer(&mut file, &obj)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum RunError {
    #[error("Failed to read file {0}: {1}")]
    Io(PathBuf, std::io::Error),

    #[error("Failed to deserialize {0}: {1}")]
    Deserialize(PathBuf, ReadJsonError),

    #[error("Internal error: failed to load bootloader")]
    FailedToLoadBootloader(ProgramError),

    #[error("Failed to load program {0}: {1}")]
    FailedToLoadProgram(PathBuf, ProgramError),

    #[error("Failed to load PIE {0}: {1}")]
    FailedToLoadPie(PathBuf, CairoPieError),

    #[error(transparent)]
    FailedExecution(#[from] ExecutionError),

    #[error(transparent)]
    Prover(#[from] ProverError),
}

pub fn run_program(args: ProveBareArgs) -> Result<ExecutionArtifacts, RunError> {
    let layout = args.layout.unwrap_or(Layout::StarknetWithKeccak);

    let program = std::fs::read(&args.program).map_err(|e| RunError::Io(args.program, e))?;
    let (runner, vm) = run_in_proof_mode(&program, layout).map_err(ExecutionError::RunFailed)?;
    extract_execution_artifacts(runner, vm).map_err(|e| e.into())
}

fn is_zip_file(file: &Path) -> bool {
    match file.extension() {
        Some(extension) => extension == "zip",
        None => false,
    }
}

#[derive(thiserror::Error, Debug)]
enum TaskError {
    #[error(transparent)]
    Pie(#[from] CairoPieError),

    #[error(transparent)]
    Program(#[from] ProgramError),
}

fn task_from_file(file: &Path) -> Result<TaskSpec, TaskError> {
    let task = if is_zip_file(file) {
        let pie = CairoPie::from_file(file)?;
        Task::Pie(pie)
    } else {
        let program = Program::from_file(file, Some("main"))?;
        Task::Program(program)
    };

    Ok(TaskSpec { task })
}

pub fn run_with_bootloader(args: ProveWithBootloaderArgs) -> Result<ExecutionArtifacts, RunError> {
    let bootloader = Program::from_bytes(BOOTLOADER_PROGRAM, Some("main"))
        .map_err(RunError::FailedToLoadBootloader)?;
    let tasks: Result<Vec<TaskSpec>, RunError> = args
        .programs
        .into_iter()
        .map(|path_buf| {
            task_from_file(path_buf.as_path()).map_err(|e| match e {
                TaskError::Pie(e) => RunError::FailedToLoadPie(path_buf, e),
                TaskError::Program(e) => RunError::FailedToLoadProgram(path_buf, e),
            })
        })
        .collect();
    let tasks = tasks?;
    run_bootloader_in_proof_mode(&bootloader, tasks).map_err(|e| e.into())
}

pub fn prove(command: ProveCommand) -> Result<(), RunError> {
    // Cloning here is the easiest solution to avoid borrow checks.
    let config_args = command.config().clone();

    let user_prover_config = config_args
        .prover_config_file
        .as_ref()
        .map(|path| read_json_from_file(path).map_err(|e| RunError::Deserialize(path.clone(), e)))
        .transpose()?;
    let prover_config = user_prover_config.unwrap_or(ProverConfig::default());

    let user_prover_parameters = config_args
        .parameter_file
        .as_ref()
        .map(|path| read_json_from_file(path).map_err(|e| RunError::Deserialize(path.clone(), e)))
        .transpose()?;

    let execution_artifacts = match command {
        ProveCommand::Bare(args) => run_program(args)?,
        ProveCommand::WithBootloader(args) => run_with_bootloader(args)?,
    };

    let last_layer_degree_bound = 64;
    let prover_parameters = user_prover_parameters.unwrap_or(generate_prover_parameters(
        execution_artifacts.public_input.n_steps,
        last_layer_degree_bound,
    ));

    let proof = run_prover(
        &execution_artifacts.public_input,
        &execution_artifacts.private_input,
        &execution_artifacts.memory,
        &execution_artifacts.trace,
        &prover_config,
        &prover_parameters,
    )?;

    let output_file = config_args.output_file();
    write_json_to_file(proof, output_file.as_ref())
        .map_err(|e| RunError::Io(output_file.into_owned(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::path::PathBuf;

    #[rstest]
    #[case("cairo_pie.zip", true)]
    #[case("program.json", false)]
    fn test_is_zip_file(#[case] file: PathBuf, #[case] expected: bool) {
        assert_eq!(is_zip_file(file.as_path()), expected);
    }
}
