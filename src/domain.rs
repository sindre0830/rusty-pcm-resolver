use std::path::PathBuf;

/// represents either a remote media url or a local file path
#[derive(Clone, Debug)]
pub enum MediaInput {
    Url(String),
    File(PathBuf),
}
