use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, Subcommand};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use stone_prover_sdk::models::Layout;

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

        if self.with_bootloader {
            let args = ProveWithBootloaderArgs {
                programs: self.programs,
                config: self.config,
                layout: self.layout,
            };
            return ProveCommand::WithBootloader(args);
        }
        let args = ProveBareArgs {
            program: self.programs.remove(0),
            program_input: self.program_input,
            layout: self.layout,
            config: self.config,
        };
        ProveCommand::Bare(args)
    }
}

#[derive(Subcommand, Debug)]
pub enum ProveCommand {
    Bare(ProveBareArgs),
    WithBootloader(ProveWithBootloaderArgs),
}

impl ProveCommand {
    pub fn config(&self) -> &ConfigArgs {
        match self {
            ProveCommand::Bare(args) => &args.config,
            ProveCommand::WithBootloader(args) => &args.config,
        }
    }
}

#[derive(Args, Debug)]
pub struct ProveBareArgs {
    pub program: PathBuf,

    #[clap(long = "program-input")]
    pub program_input: Option<PathBuf>,

    #[clap(long = "layout")]
    pub layout: Option<Layout>,

    #[clap(flatten)]
    pub config: ConfigArgs,
}

#[derive(Args, Debug)]
pub struct ProveWithBootloaderArgs {
    pub programs: Vec<PathBuf>,

    #[clap(long = "layout")]
    pub layout: Option<Layout>,

    #[clap(flatten)]
    pub config: ConfigArgs,
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
