# Rusty PCM Resolver

**Rusty PCM Resolver** is a lightweight Rust library designed to automatically detect, resolve, and decode audio from various sources into raw PCM (`f32le`) data.
It supports multiple input types (URLs, local files, or resolvable media sources) and uses [`ffmpeg`](https://www.ffmpeg.org/) for decoding, with optional integration with [`yt-dlp`](https://github.com/yt-dlp/yt-dlp).

By automating source detection, conversion, and caching, this library simplifies audio processing pipelines, making it ideal for projects involving transcription, speech recognition, or signal analysis.

## Prerequisites

* `ffmpeg` installed and accessible from the command line
* (Optional) `yt-dlp` for resolving external media URLs

---

## Usage Guide

To include this crate in your project, add it to your dependencies:

```bash
cargo add --git https://github.com/sindre0830/rusty-pcm-resolver.git --tag v1.0.0 rusty-pcm-resolver
```

Or manually in your `Cargo.toml`:

```toml
[dependencies]
rusty-pcm-resolver = { git = "https://github.com/sindre0830/rusty-pcm-resolver.git", tag = "v1.0.0" }
```

### Example

```rust
use anyhow::Result;

use rusty_pcm_resolver::domain::MediaInput;
use rusty_pcm_resolver::{Options, PcmResolver};

fn main() -> Result<()> {
    // example 1: resolve, decode, and load pcm into memory from a remote media URL
    let options_1 = Options::new(MediaInput::Url("https://url.to/audio".into()));
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
```

---

## API Overview

### Core Functions

| Method                      | Input     | Output             | Description                                                                                                |
| --------------------------- | --------- | ------------------ | ---------------------------------------------------------------------------------------------------------- |
| `PcmResolver::new(options)` | `Options` | `PcmResolver`      | Creates a new resolver pipeline configured with `Options`.                                                 |
| `resolve_media()`           | —         | `Self`             | Resolves the `MediaInput` inside `Options` (e.g., validates local file or runs `yt-dlp`). |
| `convert_to_pcm()`          | —         | `Self`             | Converts the resolved media into PCM (`f32le`) using `ffmpeg`. Caches the result.        |
| `load()`                    | —         | `Result<Vec<f32>>` | Loads decoded PCM samples from the cached file into memory.                                                |
| `into_path()`               | —         | `Result<PathBuf>`  | Returns the cached PCM file path without loading samples.                                                  |

Each pipeline step consumes the previous state to enforce the correct order (resolve → convert → load).
If the PCM already exists in the cache, convert_to_pcm() will skip reprocessing automatically.

---

### Configuration

| Type                  | Description                                                                                                                                                            |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`Options`**         | Configuration object describing what to process and how. Holds `MediaInput`, `sample_rate_hz`, `channels`, optional `referer`, and `cache_dir`.                                     |
| **`Options::hash()`** | Returns a deterministic hash ID for the combination of `MediaInput`, `sample_rate_hz`, `channels`, and `referer`. Useful for naming related files (e.g., transcripts). |

```rust
let opts = Options::new(MediaInput::Url("https://url.to/audio".into()))
    .sample_rate(16_000)
    .channels(1);

println!("Cache ID: {}", opts.hash());
```

### Types

| Type                        | Description                                       |
| --------------------------- | ------------------------------------------------- |
| `MediaInput::Url(String)`   | Remote media URL (e.g., `.mp4`, `.m3u8`, `.wav`). |
| `MediaInput::File(PathBuf)` | Local file path to an audio or video file.        |

---

## Legal Notice

This project does **not** include or distribute `yt-dlp`.
Integration with `yt-dlp` is optional and requires separate installation.
Users are solely responsible for ensuring that their usage complies with all applicable laws and the Terms of Service of the websites they access.

---

## Development Guide

### Commands

| Command            | Description                             | Example                                                    |
| ------------------ | --------------------------------------- | ---------------------------------------------------------- |
| **Build**          | Compiles the crate in release mode      | `cargo build --release`                                    |
| **Run Tests**      | Executes all unit tests                 | `cargo test`                                               |
| **Lint (Clippy)**  | Checks for style and performance issues | `cargo clippy --all-targets --all-features -- -D warnings` |
| **Format Code**    | Formats the entire codebase             | `cargo fmt`                                                |
| **Doc Generation** | Builds local documentation              | `cargo doc --open`                                         |

---

### Upgrading Dependencies

To upgrade all dependencies to the latest compatible versions:

```bash
cargo update
```

Or for a specific crate:

```bash
cargo update -p crate-name
```
