use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;

use crate::domain::MediaInput;
use crate::resolver::traits::MediaResolver;

pub struct YouTubeResolver;

impl MediaResolver for YouTubeResolver {
    fn name(&self) -> &'static str {
        "youtube"
    }

    fn matches(&self, url: &Url) -> bool {
        matches!(url.domain(), Some(d) if d.contains("youtube.com") || d.contains("youtu.be"))
    }

    fn resolve(&self, original: &str, cache_dir: &Path) -> Result<Option<MediaInput>> {
        let cache_dir = &cache_dir.join("ytdlp");
        fs::create_dir_all(cache_dir).context("failed to create ytdlp cache dir")?;
        let stem = blake3::hash(original.as_bytes()).to_hex().to_string();

        if let Some(p) = find_cached_by_stem(cache_dir, &stem)? {
            return Ok(Some(MediaInput::File(p)));
        }

        let template = cache_dir.join(format!("{stem}.%(ext)s"));
        let status = Command::new("yt-dlp")
            .args([
                "--quiet",
                "--no-warnings",
                "--no-playlist",
                "--no-overwrites",
                "--no-part",
                "-f",
                "bestaudio",
                "-o",
                &template.to_string_lossy(),
                original,
            ])
            .status()
            .context("failed to run yt-dlp")?;

        if !status.success() {
            return Err(anyhow!("yt-dlp download failed: {}", status));
        }

        let downloaded = find_cached_by_stem(cache_dir, &stem)?
            .ok_or_else(|| anyhow!("yt-dlp reported success but output file not found"))?;

        Ok(Some(MediaInput::File(downloaded)))
    }
}

fn find_cached_by_stem(dir: &Path, stem: &str) -> Result<Option<PathBuf>> {
    for entry in fs::read_dir(dir).with_context(|| format!("listing {}", dir.display()))? {
        let p = entry?.path();
        if let Some(ext) = p.extension().and_then(|e| e.to_str())
            && (ext.eq_ignore_ascii_case("part") || ext.eq_ignore_ascii_case("tmp"))
        {
            continue;
        }
        if p.file_stem().and_then(|s| s.to_str()) == Some(stem) && p.is_file() {
            return Ok(Some(p));
        }
    }
    Ok(None)
}
