use crate::cli::Cli;
use crate::commands::hash::HashError;
use crate::commands::prove::RunError;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use clap::Parser;
use env_logger::fmt::Formatter;
use log::{error, LevelFilter, Record};
use std::io;
use std::io::Write;
use stone_prover_sdk::cairo_vm::ExecutionError;
use stone_prover_sdk::error::VerifierError;

mod cli;
mod commands;
mod toolkit;

#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error(transparent)]
    Hash(#[from] HashError),
    #[error(transparent)]
    Prove(#[from] RunError),
    #[error(transparent)]
    Verify(#[from] VerifierError),
}

fn format_log(buf: &mut Formatter, record: &Record) -> io::Result<()> {
    let level_style = buf.default_level_style(record.level());
    writeln!(
        buf,
        "{level_style}{:<5}{level_style:#} {}",
        record.level().to_string().to_lowercase(),
        record.args()
    )
}

fn setup_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .format(format_log)
        .init();
}

fn display_error(error: CliError) {
    let error_message = match error {
        CliError::Hash(hash_error) => hash_error.to_string(),
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
    error!("{}", error_message);
}

fn process_cli_command(command: Cli) -> Result<(), CliError> {
    match command {
        Cli::Hash(hash_args) => commands::hash(hash_args.program)?,
        Cli::Prove(prove_args) => commands::prove(prove_args.command())?,
        Cli::Verify(verify_args) => commands::verify(verify_args)?,
    };

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    let command = Cli::parse();
    if let Err(e) = process_cli_command(command) {
        display_error(e);
    }

    Ok(())
}
