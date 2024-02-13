use serde::de::DeserializeOwned;
use std::fs::File;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum ReadJsonError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn read_json_from_file<T: DeserializeOwned, P: AsRef<Path>>(
    path: P,
) -> Result<T, ReadJsonError> {
    let file = File::open(path)?;
    let mut reader = std::io::BufReader::new(file);

    let obj: T = serde_json::from_reader(&mut reader)?;
    Ok(obj)
}
