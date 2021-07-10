use std::path::PathBuf;
use tempfile::{TempPath, NamedTempFile};

pub fn cargo_manifest_path() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "Cargo.toml"].iter().collect()
}

pub fn create_temp_path() -> TempPath {
    NamedTempFile::new()
        .expect("Cannot open a temporary file")
        .into_temp_path()
}
