use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use clap::Parser;
use stone_prover_sdk::cairo_vm::ExecutionError;
use stone_prover_sdk::error::VerifierError;

use crate::cli::Cli;
use crate::commands::prove::RunError;

mod cli;
mod commands;
mod toolkit;

#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error(transparent)]
    Prove(#[from] RunError),
    #[error(transparent)]
    Verify(#[from] VerifierError),
}

fn display_error(error: CliError) {
    let error_message = match error {
        CliError::Prove(run_error) => match run_error {
            RunError::Io(path_buf, io_error) => {
                format!("could not read {}: {io_error}.", path_buf.to_string_lossy())
            }
            RunError::Deserialize(path_buf, json_error) => {
                format!(
                    "could not read JSON file {}: {json_error}.",
                    path_buf.to_string_lossy()
                )
            }
            RunError::FailedToLoadBootloader(program_error) => {
                format!("failed to load bootloader program: {program_error}. This is an internal error and should not happen.")
            }
            RunError::FailedToLoadProgram(path_buf, program_error) => {
                format!(
                    "failed to load program {}: {program_error}.",
                    path_buf.to_string_lossy()
                )
            }
            RunError::FailedToLoadPie(path_buf, pie_error) => {
                format!(
                    "failed to load Cairo PIE {}: {pie_error}.",
                    path_buf.to_string_lossy()
                )
            }
            RunError::FailedExecution(execution_error) => match execution_error {
                ExecutionError::RunFailed(cairo_run_error) => match cairo_run_error {
                    CairoRunError::Program(program_error) => {
                        format!("failed to load program: {program_error}")
                    }
                    other => format!("failed to run Cairo program: {other}"),
                },
                other => format!("failed to extract VM output(s): {other}"),
            },
            RunError::Prover(prover_error) => {
                format!("failed to run prover: {prover_error}")
            }
        },
        CliError::Verify(e) => match e {
            VerifierError::IoError(_) => {
                "could not find verifier program. Is cpu_air_verifier installed?".to_string()
            }
            VerifierError::CommandError(command_output) => {
                format!(
                    "failed to run verifier: {}",
                    String::from_utf8_lossy(&command_output.stderr)
                )
            }
        },
    };
    println!("Error: {}", error_message);
}

fn process_cli_command(command: Cli) -> Result<(), CliError> {
    match command {
        Cli::Prove(prove_args) => commands::prove(prove_args.command())?,
        Cli::Verify(verify_args) => commands::verify(verify_args)?,
    };

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Cli::parse();
    if let Err(e) = process_cli_command(command) {
        display_error(e);
    }

    Ok(())
}
