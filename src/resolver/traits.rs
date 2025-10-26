use anyhow::Result;
use std::path::Path;
use url::Url;

use crate::domain::MediaInput;

/// user-extensible resolver interface
pub trait MediaResolver: Send + Sync {
    /// short name for logging
    fn name(&self) -> &'static str;

    /// whether this resolver should handle the url
    fn matches(&self, url: &Url) -> bool;

    /// resolve into a concrete media input (file/url) or return None to defer
    fn resolve(&self, original: &str, cache_dir: &Path) -> Result<Option<MediaInput>>;
}
