# Rusty PCM Resolver

A lightweight Rust library that automatically detects, resolves, and decodes audio from various sources into raw PCM (`f32le`) data.

## Features

- Detects and resolves media links (e.g., direct audio/video URLs, file paths, or external resolvers).
- Converts any supported audio source to PCM using `ffmpeg`.
- Optionally integrates with [`yt-dlp`](https://github.com/yt-dlp/yt-dlp).

## Legal Notice

This project does **not** include or distribute `yt-dlp`.  
Integration with `yt-dlp` is optional and requires it to be installed separately by the user.  
Users are solely responsible for ensuring that their usage of this software complies with all applicable laws and the Terms of Service of the websites they access.

## API Overview

``resolve_pcm()`` function uses the other functions.

| Function | Input | Output | Description |
|-----------|--------|---------|-------------|
| `resolve_pcm(media_input, referer, sample_rate_hz, channels)` | `MediaInput`, `Option<&str>`, `Option<u32>`, `Option<u8>` | `Result<Vec<f32>>` | Resolves a media source (URL or file) and returns decoded PCM samples (`f32le`). Automatically caches conversions. |
| `resolve_media_input(media_input)` | `MediaInput` | `Result<MediaInput>` | Validates or resolves a media input through internal resolvers (e.g. yt-dlp). |
| `download_pcm_from_source(media, referer, sample_rate_hz, channels)` | `&MediaInput`, `Option<&str>`, `Option<u32>`, `Option<u8>` | `Result<PathBuf>` | Downloads or converts media to PCM and stores the cached file on disk. |
| `load_pcm_from_path(path)` | `&Path` | `Result<Vec<f32>>` | Reads an existing PCM file into memory as `Vec<f32>`. |

---

### Types

| Type | Description |
|------|--------------|
| `MediaInput::Url(String)` | Remote media URL (e.g. `.mp4`, `.m3u8`, or other stream). |
| `MediaInput::File(PathBuf)` | Local file path to audio or video file. |

---

Each function uses `ffmpeg` for decoding and caches results in `./.cache/pcm`.

## Setup

Add crate to your project (change tag to latest release):

```bash
cargo add --git https://github.com/sindre0830/rusty-pcm-resolver.git --tag v0.1.0 rusty_pcm_resolver
```

Requires ffmpeg installed on the system
