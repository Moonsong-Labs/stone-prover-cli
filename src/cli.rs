use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use stone_prover_sdk::models::{Layout, Verifier};

#[derive(Parser, Debug)]
#[command(name = "stone")]
#[command(bin_name = "stone")]
pub enum Cli {
    Prove(ProveArgs),
    Verify(VerifyArgs),
}

#[derive(Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ProveArgs {
    #[clap(long = "with-bootloader", default_value_t = false)]
    pub with_bootloader: bool,

    #[clap(long = "program-input")]
    pub program_input: Option<PathBuf>,

    #[clap(long = "layout")]
    pub layout: Option<Layout>,

    #[clap(long = "verifier")]
    pub verifier: Option<Verifier>,

    #[clap(long = "allow-missing-builtins", action)]
    pub allow_missing_builtins: bool,

    #[clap(flatten)]
    pub config: ConfigArgs,

    #[arg(required = true, num_args = 1..)]
    pub programs: Vec<PathBuf>,
}

impl ProveArgs {
    pub fn command(mut self) -> ProveCommand {
        let mut cmd = Cli::command();
        if self.with_bootloader {
            if self.program_input.is_some() {
                cmd.error(
                    ErrorKind::ArgumentConflict,
                    "Cannot load program input in bootloader mode",
                )
                .exit();
            }
        } else if self.programs.len() > 1 {
            cmd.error(
                ErrorKind::ArgumentConflict,
                "Cannot prove multiple programs without bootloader",
            )
            .exit();
        }

        let executable = match self.with_bootloader {
            true => Executable::WithBootloader(self.programs),
            false => Executable::BareMetal(self.programs.remove(0)),
        };
        let layout = self.layout.unwrap_or(Layout::StarknetWithKeccak);
        let verifier = self.verifier.unwrap_or(Verifier::Stone);

        ProveCommand {
            executable,
            config: self.config,
            layout,
            verifier,
            allow_missing_builtins: self.allow_missing_builtins,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProveCommand {
    pub executable: Executable,
    pub config: ConfigArgs,
    pub layout: Layout,
    pub verifier: Verifier,
    pub allow_missing_builtins: bool,
}

#[derive(Debug, Clone)]
pub enum Executable {
    BareMetal(PathBuf),
    WithBootloader(Vec<PathBuf>),
}

#[derive(Args, Clone, Debug)]
pub struct ConfigArgs {
    #[clap(long = "prover-config-file")]
    pub prover_config_file: Option<PathBuf>,
    #[clap(long = "parameter-file")]
    pub parameter_file: Option<PathBuf>,
    #[clap(long = "output-file")]
    output_file: Option<PathBuf>,
}

impl ConfigArgs {
    pub fn output_file(&self) -> Cow<PathBuf> {
        match self.output_file.as_ref() {
            Some(path) => Cow::Borrowed(path),
            None => Cow::Owned(Path::new("proof.json").to_path_buf()),
        }
    }
}

#[derive(Args, Clone, Debug)]
pub struct VerifyArgs {
    pub proof_file: PathBuf,
}
