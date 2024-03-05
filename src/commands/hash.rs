use std::path::{Path, PathBuf};

use cairo_vm::types::errors::cairo_pie_error::CairoPieError;
use cairo_vm::types::errors::program_errors::ProgramError;
use cairo_vm::types::program::Program;
use cairo_vm::vm::runners::cairo_pie::{CairoPie, StrippedProgram};

use crate::toolkit::program_hash::{compute_program_hash_chain, ProgramHashError};
use crate::toolkit::zip::is_zip_file;

#[derive(thiserror::Error, Debug)]
pub enum HashError {
    #[error(transparent)]
    CairoPie(#[from] CairoPieError),
    #[error(transparent)]
    Program(#[from] ProgramError),
    #[error(transparent)]
    ProgramHash(#[from] ProgramHashError),
}

fn stripped_program_from_file(file: &Path) -> Result<StrippedProgram, HashError> {
    let stripped_program = if is_zip_file(file) {
        let pie = CairoPie::from_file(file)?;
        pie.metadata.program
    } else {
        let program = Program::from_file(file, Some("main"))?;
        program.get_stripped_program()?
    };

    Ok(stripped_program)
}

pub fn hash(program_path: PathBuf) -> Result<(), HashError> {
    let program = stripped_program_from_file(&program_path)?;
    let bootloader_version = 0;

    let program_hash = compute_program_hash_chain(&program, bootloader_version)?;
    // Note that we use print here to avoid additional output and make the command
    // usable in a script without parsing the output.
    println!("{:#x}", program_hash);

    Ok(())
}
