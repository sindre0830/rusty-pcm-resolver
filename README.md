# Rusty PCM Resolver

**Rusty PCM Resolver** is a lightweight Rust library designed to automatically detect, resolve, and decode audio from various sources into raw PCM (`f32le`) data.
It supports multiple input types (URLs, local files, or resolvable media sources) and uses [`ffmpeg`](https://www.ffmpeg.org/) for decoding, with optional integration with [`yt-dlp`](https://github.com/yt-dlp/yt-dlp).

By automating source detection, conversion, and caching, this library simplifies audio processing pipelines, making it ideal for projects involving transcription, speech recognition, or signal analysis.

---

## Usage Guide

To include this crate in your project, add it to your dependencies:

```bash
cargo add --git https://github.com/sindre0830/rusty-pcm-resolver.git --tag v0.1.0 rusty-pcm-resolver
```

Or manually in your `Cargo.toml`:

```toml
[dependencies]
rusty-pcm-resolver = { git = "https://github.com/sindre0830/rusty-pcm-resolver.git", tag = "v0.1.0" }
```

Make sure `ffmpeg` is installed and accessible in your system’s PATH.

### Example

```rust
// example 1: resolve and decode a remote media URL
let remote_url = "https://url.to/audio";
let remote_media = MediaInput::Url(remote_url.to_string());

let remote_samples = resolve_pcm(remote_media, None, None, None)?;
println!("Loaded {} samples from remote URL", remote_samples.len());
```

```rust
// example 2: resolve and decode from a local file path
let local_path = PathBuf::from("path/to/local/file.something");
let local_media = MediaInput::File(local_path);

let local_samples = resolve_pcm(local_media, None, None, None)?;
println!("Loaded {} samples from local file", local_samples.len());
```

---

## API Overview

### Core Functions

| Function                                                             | Input                                                      | Output               | Description                                                                                                        |
| -------------------------------------------------------------------- | ---------------------------------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `resolve_pcm(media_input, referer, sample_rate_hz, channels)`        | `MediaInput`, `Option<&str>`, `Option<u32>`, `Option<u8>`  | `Result<Vec<f32>>`   | Resolves a media source (URL or file) and returns decoded PCM samples (`f32le`). Automatically caches conversions. |
| `resolve_media_input(media_input)`                                   | `MediaInput`                                               | `Result<MediaInput>` | Validates or resolves a media input through internal resolvers (e.g., `yt-dlp`).                                   |
| `download_pcm_from_source(media, referer, sample_rate_hz, channels)` | `&MediaInput`, `Option<&str>`, `Option<u32>`, `Option<u8>` | `Result<PathBuf>`    | Downloads or converts media to PCM and stores the cached file on disk.                                             |
| `load_pcm_from_path(path)`                                           | `&Path`                                                    | `Result<Vec<f32>>`   | Loads an existing PCM file from disk into memory.                                                                  |

`resolve_pcm` serves as the main entry point, internally calling the other three functions to resolve, download, and load the PCM data.

---

### Types

| Type                        | Description                                       |
| --------------------------- | ------------------------------------------------- |
| `MediaInput::Url(String)`   | Remote media URL (e.g., `.mp4`, `.m3u8`, `.wav`). |
| `MediaInput::File(PathBuf)` | Local file path to an audio or video file.        |

All conversions are cached under `./.cache/pcm` for faster repeated access.

---

## Legal Notice

This project does **not** include or distribute `yt-dlp`.
Integration with `yt-dlp` is optional and requires separate installation.
Users are solely responsible for ensuring that their usage complies with all applicable laws and the Terms of Service of the websites they access.

---

## Development Guide

### Prerequisites

* `ffmpeg` installed and accessible from the command line
* (Optional) `yt-dlp` for resolving external media URLs

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
