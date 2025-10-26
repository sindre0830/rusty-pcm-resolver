use anyhow::Result;

use rusty_pcm_resolver::PcmResolver;

fn main() -> Result<()> {
    // example 1: resolve, decode, and load into pcm into memory from a remote media URL
    let remote_samples = PcmResolver::new(rusty_pcm_resolver::Options::default())
        .resolve_media(rusty_pcm_resolver::domain::MediaInput::Url(
            "https://www.youtube.com/watch?v=rqN3S6ZOBRU".into(),
        ))?
        .convert_to_pcm()?
        .load()?;
    println!("Loaded {} samples from remote URL", remote_samples.len());

    // example 2: resolve, decode, and load into pcm into memory from a local file path
    let local_samples = PcmResolver::new(rusty_pcm_resolver::Options::default())
        .resolve_media(rusty_pcm_resolver::domain::MediaInput::File(
            ".cache/ytdlp/8c48d911867c5fe8bd20e4665758ef446282c2c0de4efabb703b92bf61f04a9e.webm"
                .into(),
        ))?
        .convert_to_pcm()?
        .load()?;
    println!("Loaded {} samples from local file", local_samples.len());

    // example 3: resolve and decode a remote media URL (just return PCM path)
    let remote_pcm_path = PcmResolver::new(rusty_pcm_resolver::Options::default())
        .resolve_media(rusty_pcm_resolver::domain::MediaInput::Url(
            "https://www.youtube.com/watch?v=rqN3S6ZOBRU".into(),
        ))?
        .convert_to_pcm()?
        .into_path()?;
    println!("PCM cached at {}", remote_pcm_path.display());

    Ok(())
}
