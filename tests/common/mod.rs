use rstest::fixture;
use std::path::{Path, PathBuf};

/// Inspired by the `test_bin` crate
fn get_target_dir() -> PathBuf {
    // Cargo puts the integration test binary in target/<buildmode>/deps
    let current_exe = std::env::current_exe().unwrap();
    let build_dir = current_exe.parent().unwrap().parent().unwrap();

    build_dir.to_path_buf()
}

#[fixture]
pub fn cli_in_path() {
    // Add build dir to path for the duration of the test
    let path = std::env::var("PATH").unwrap_or_default();

    // The prover and verifier are downloaded in dependencies by the Makefile
    let deps_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("dependencies");
    // The final binary is located in the target directory
    let target_dir = get_target_dir();

    std::env::set_var(
        "PATH",
        format!(
            "{}:{}:{path}",
            deps_dir.to_string_lossy(),
            target_dir.to_string_lossy()
        ),
    );
}
