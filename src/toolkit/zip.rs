use std::path::Path;

pub fn is_zip_file(file: &Path) -> bool {
    match file.extension() {
        Some(extension) => extension == "zip",
        None => false,
    }
}
