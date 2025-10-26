use anyhow::Result;

use rusty_pcm_resolver::domain::MediaInput;
use rusty_pcm_resolver::{Options, PcmResolver};

fn main() -> Result<()> {
    // example 1: resolve, decode, and load pcm into memory from a remote media URL
    let options_1 = Options::new(MediaInput::Url("https://url.to/audio".into()));

    println!(
        "This is the filename that the pcm file will be cached as: {}",
        options_1.hash()
    );

    let remote_samples = PcmResolver::new(options_1)
        .resolve_media()?
        .convert_to_pcm()?
        .load()?;
    println!("Loaded {} samples from remote URL", remote_samples.len());

    // example 2: resolve, decode, and load pcm into memory from a local file path
    let options_2 = Options::new(MediaInput::File("path/to/local/file.something".into()));
    let local_samples = PcmResolver::new(options_2)
        .resolve_media()?
        .convert_to_pcm()?
        .load()?;
    println!("Loaded {} samples from local file", local_samples.len());

    // example 3: resolve and decode a remote media URL (returns cached PCM path)
    let options_3 = Options::new(MediaInput::Url("https://url.to/audio".into()));
    let remote_pcm_path = PcmResolver::new(options_3)
        .resolve_media()?
        .convert_to_pcm()?
        .into_path()?;
    println!("PCM cached at {}", remote_pcm_path.display());

    Ok(())
}
