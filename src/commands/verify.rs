use log::info;
use stone_prover_sdk::error::VerifierError;
use stone_prover_sdk::verifier::run_verifier;

use crate::cli::VerifyArgs;

pub fn verify(args: VerifyArgs) -> Result<(), VerifierError> {
    info!("verification in progress...");
    run_verifier(args.proof_file.as_path())?;
    info!("verification completed!");

    Ok(())
}
