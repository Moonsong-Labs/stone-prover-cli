// TODO: replace by the `cairo-vm` implementation once
//       https://github.com/lambdaclass/cairo-vm/pull/1647 is merged.

use cairo_vm::serde::deserialize_program::BuiltinName;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::runners::cairo_pie::StrippedProgram;
use cairo_vm::Felt252;
use starknet_crypto::{pedersen_hash, FieldElement};

type HashFunction = fn(&FieldElement, &FieldElement) -> FieldElement;

#[derive(thiserror::Error, Debug)]
pub enum HashChainError {
    #[error("Data array must contain at least one element.")]
    EmptyData,
}

#[derive(thiserror::Error, Debug)]
pub enum ProgramHashError {
    #[error(transparent)]
    HashChain(#[from] HashChainError),

    #[error(
    "Invalid program builtin: builtin name too long to be converted to field element: {0}"
    )]
    InvalidProgramBuiltin(&'static str),

    #[error("Invalid program data: data contains relocatable(s)")]
    InvalidProgramData,

    /// Conversion from Felt252 to FieldElement failed. This is unlikely to happen
    /// unless the implementation of Felt252 changes and this code is not updated properly.
    #[error("Conversion from Felt252 to FieldElement failed")]
    Felt252ToFieldElementConversionFailed,
}

/// Computes a hash chain over the data, in the following order:
///     h(data[0], h(data[1], h(..., h(data[n-2], data[n-1])))).
///
/// Reimplements this Python function:
/// def compute_hash_chain(data, hash_func=pedersen_hash):
///     assert len(data) >= 1, f"len(data) for hash chain computation must be >= 1; got: {len(data)}."
///     return functools.reduce(lambda x, y: hash_func(y, x), data[::-1])
fn compute_hash_chain<'a, I>(
    data: I,
    hash_func: HashFunction,
) -> Result<FieldElement, HashChainError>
    where
        I: Iterator<Item=&'a FieldElement> + DoubleEndedIterator,
{
    match data.copied().rev().reduce(|x, y| hash_func(&y, &x)) {
        Some(result) => Ok(result),
        None => Err(HashChainError::EmptyData),
    }
}

/// Creates an instance of `FieldElement` from a builtin name.
///
/// Converts the builtin name to bytes then attempts to create a field element from
/// these bytes. This function will fail if the builtin name is over 31 characters.
fn builtin_to_field_element(builtin: &BuiltinName) -> Result<FieldElement, ProgramHashError> {
    // The Python implementation uses the builtin name without suffix
    let builtin_name = builtin
        .name()
        .strip_suffix("_builtin")
        .unwrap_or(builtin.name());

    FieldElement::from_byte_slice_be(builtin_name.as_bytes())
        .map_err(|_| ProgramHashError::InvalidProgramBuiltin(builtin.name()))
}

/// The `value: FieldElement` is `pub(crate)` and there is no accessor.
/// This function converts a `Felt252` to a `FieldElement` using a safe, albeit inefficient,
/// method.
fn felt_to_field_element(felt: &Felt252) -> Result<FieldElement, ProgramHashError> {
    let bytes = felt.to_bytes_be();
    FieldElement::from_bytes_be(&bytes)
        .map_err(|_e| ProgramHashError::Felt252ToFieldElementConversionFailed)
}

/// Converts a `MaybeRelocatable` into a `FieldElement` value.
///
/// Returns `InvalidProgramData` if `maybe_relocatable` is not an integer
fn maybe_relocatable_to_field_element(
    maybe_relocatable: &MaybeRelocatable,
) -> Result<FieldElement, ProgramHashError> {
    let felt = maybe_relocatable
        .get_int_ref()
        .ok_or(ProgramHashError::InvalidProgramData)?;
    felt_to_field_element(felt)
}

/// Computes the Pedersen hash of a program.
///
/// Reimplements this Python function:
/// def compute_program_hash_chain(program: ProgramBase, bootloader_version=0):
///     builtin_list = [from_bytes(builtin.encode("ascii")) for builtin in program.builtins]
///     # The program header below is missing the data length, which is later added to the data_chain.
///     program_header = [bootloader_version, program.main, len(program.builtins)] + builtin_list
///     data_chain = program_header + program.data
///
///     return compute_hash_chain([len(data_chain)] + data_chain)
pub fn compute_program_hash_chain(
    program: &StrippedProgram,
    bootloader_version: usize,
) -> Result<FieldElement, ProgramHashError> {
    let program_main = program.main;
    let program_main = FieldElement::from(program_main);

    // Convert builtin names to field elements
    let builtin_list: Result<Vec<FieldElement>, _> = program
        .builtins
        .iter()
        .map(builtin_to_field_element)
        .collect();
    let builtin_list = builtin_list?;

    let program_header = vec![
        FieldElement::from(bootloader_version),
        program_main,
        FieldElement::from(program.builtins.len()),
    ];

    let program_data: Result<Vec<_>, _> = program
        .data
        .iter()
        .map(maybe_relocatable_to_field_element)
        .collect();
    let program_data = program_data?;

    let data_chain_len = program_header.len() + builtin_list.len() + program_data.len();
    let data_chain_len_vec = vec![FieldElement::from(data_chain_len)];

    // Prepare a chain of iterators to feed to the hash function
    let data_chain = [
        &data_chain_len_vec,
        &program_header,
        &builtin_list,
        &program_data,
    ];

    let hash = compute_hash_chain(data_chain.iter().flat_map(|&v| v.iter()), pedersen_hash)?;
    Ok(hash)
}
