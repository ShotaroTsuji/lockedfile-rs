use tempfile::{TempPath, NamedTempFile};

pub fn create_temp_path() -> TempPath {
    NamedTempFile::new()
        .expect("Cannot open a temporary file")
        .into_temp_path()
}
