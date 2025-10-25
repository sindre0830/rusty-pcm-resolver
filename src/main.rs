use anyhow::Result;
use std::path::PathBuf;

use rusty_pcm_resolver::{MediaInput, resolve_pcm};

fn main() -> Result<()> {
    // example 1: resolve and decode a remote media URL
    let remote_url = "https://url.to/audio";
    let remote_media = MediaInput::Url(remote_url.to_string());

    let remote_samples = resolve_pcm(remote_media, None, None, None)?;
    println!("Loaded {} samples from remote URL", remote_samples.len());

    // example 2: resolve and decode from a local file path
    let local_path = PathBuf::from("path/to/local/file.something");
    let local_media = MediaInput::File(local_path);

    let local_samples = resolve_pcm(local_media, None, None, None)?;
    println!("Loaded {} samples from local file", local_samples.len());

    Ok(())
}
