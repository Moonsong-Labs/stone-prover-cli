use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use stone_prover_sdk::models::{Layout, Verifier};

#[derive(Parser, Debug)]
#[command(name = "stone")]
#[command(bin_name = "stone")]
pub enum Cli {
    Prove(ProveArgs),
    Verify(VerifyArgs),
}

#[derive(Debug, Clone)]
pub enum Bootloader {
    V0_12_3,
    V0_13_0,
    Custom(PathBuf),
}

impl Bootloader {
    pub fn latest() -> Self {
        Self::V0_13_0
    }
    pub fn latest_compatible(verifier: &Verifier) -> Self {
        match verifier {
            Verifier::Stone => Self::latest(),
            Verifier::L1 => Self::V0_12_3,
        }
    }
}

impl FromStr for Bootloader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bootloader = match s {
            "0.12.3" => Self::V0_12_3,
            "0.13.0" => Self::V0_13_0,
            path => Self::Custom(PathBuf::from(path)),
        };

        Ok(bootloader)
    }
}

#[derive(Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ProveArgs {
    #[clap(long = "with-bootloader", default_value_t = false)]
    pub with_bootloader: bool,

    #[clap(long = "bootloader-version")]
    bootloader: Option<Bootloader>,

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
        if !self.with_bootloader {
            if self.config.fact_topologies_file.is_some() {
                cmd.error(
                    ErrorKind::ArgumentConflict,
                    "Cannot specify fact topologies file in no-bootloader mode",
                )
                .exit();
            }
            if self.bootloader.is_some() {
                cmd.error(
                    ErrorKind::ArgumentConflict,
                    "Cannot specify bootloader version in no-bootloader mode",
                )
                .exit();
            }
            if self.programs.len() > 1 {
                cmd.error(
                    ErrorKind::ArgumentConflict,
                    "Cannot prove multiple programs in no-bootloader mode",
                )
                .exit();
            }
        }

        let layout = self.layout.unwrap_or(Layout::StarknetWithKeccak);
        let verifier = self.verifier.unwrap_or(Verifier::Stone);

        let executable = match self.with_bootloader {
            true => {
                let bootloader = match self.bootloader {
                    Some(version) => version,
                    None => Bootloader::latest_compatible(&verifier),
                };
                Executable::WithBootloader(bootloader, self.programs)
            }
            false => Executable::BareMetal(self.programs.remove(0)),
        };

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
    WithBootloader(Bootloader, Vec<PathBuf>),
}

#[derive(Args, Clone, Debug)]
pub struct ConfigArgs {
    #[clap(long = "prover-config-file")]
    pub prover_config_file: Option<PathBuf>,
    #[clap(long = "parameter-file")]
    pub parameter_file: Option<PathBuf>,
    #[clap(long = "output-file")]
    pub output_file: Option<PathBuf>,
    #[clap(long = "fact-topologies-file")]
    pub fact_topologies_file: Option<PathBuf>,
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
