use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;

/// Resolves a media input and converts it to PCM audio samples.
///
/// The audio is converted to the specified format and cached for future requests.
///
/// # Parameters
/// - `media_input`: The media source (URL or file path)
/// - `referer`: Optional HTTP referer header (default: derived from URL or None)
/// - `sample_rate_hz`: Target sample rate in Hz (default: 16,000 Hz)
/// - `channels`: Number of audio channels (default: 1 - mono)
///
/// # Returns
/// A vector of f32.
pub fn resolve_pcm(
    media_input: MediaInput,
    referer: Option<&str>,
    sample_rate_hz: Option<u32>,
    channels: Option<u8>,
) -> Result<Vec<f32>> {
    let media = resolve_media_input(media_input)?;
    let pcm_path = download_pcm_from_source(&media, referer, sample_rate_hz, channels)?;
    let samples = load_pcm_from_path(Path::new(&pcm_path))?;
    Ok(samples)
}

/// represents either a remote media url or a local file path
pub enum MediaInput {
    Url(String),
    File(PathBuf),
}

trait MediaResolver {
    /// return a short name for logging/debugging
    fn name(&self) -> &'static str;

    /// tell if this resolver applies to the given url
    fn matches(&self, url: &Url) -> bool;

    /// return Some(media input) if resolved, None to defer to next resolver
    fn resolve(&self, original: &str) -> Result<Option<MediaInput>>;
}

struct YouTubeResolver;

impl MediaResolver for YouTubeResolver {
    fn name(&self) -> &'static str {
        "youtube"
    }

    fn matches(&self, url: &Url) -> bool {
        matches!(url.domain(), Some(d) if d.contains("youtube.com") || d.contains("youtu.be"))
    }

    fn resolve(&self, original: &str) -> Result<Option<MediaInput>> {
        let cache_dir = Path::new("./.cache/ytdlp");
        fs::create_dir_all(cache_dir).context("failed to create ytdlp cache dir")?;

        // stable file stem by hashing original url
        let stem = blake3::hash(original.as_bytes()).to_hex().to_string();

        // 1) check cache *before* invoking yt-dlp (avoid re-download)
        if let Some(p) = find_cached_by_stem(cache_dir, &stem)? {
            return Ok(Some(MediaInput::File(p)));
        }

        // unknown extension ahead; set template with the stem
        let template = cache_dir.join(format!("{stem}.%(ext)s"));
        let template_str = template.to_string_lossy().into_owned();

        // assemble yt-dlp args
        let mut args = vec![
            "--quiet".into(),
            "--no-warnings".into(),
            "--no-playlist".into(),
            "--no-overwrites".into(),
            "--no-part".into(),
            "-f".into(),
            "bestaudio".into(),
            "-o".into(),
            template_str,
            original.into(),
        ];

        // optional: parallel downloader via aria2c (opt-in with env var)
        if std::env::var("YTDLP_USE_ARIA2C").ok().as_deref() == Some("1") {
            args.splice(
                1..1,
                [
                    "--downloader".into(),
                    "aria2c".into(),
                    "--downloader-args".into(),
                    "aria2c:-x16 -s16 -k1M --summary-interval=0".into(),
                ],
            );
        }

        let status = Command::new("yt-dlp")
            .args(&args)
            .status()
            .context("failed to run yt-dlp")?;

        if !status.success() {
            return Err(anyhow!("yt-dlp download failed with status {}", status));
        }

        // 2) locate downloaded file (stem + unknown extension)
        let downloaded = find_cached_by_stem(cache_dir, &stem)?
            .ok_or_else(|| anyhow!("yt-dlp reported success but output file not found"))?;

        Ok(Some(MediaInput::File(downloaded)))
    }
}

fn find_cached_by_stem(dir: &Path, stem: &str) -> Result<Option<PathBuf>> {
    // look for any file whose file_stem equals our stem, ignoring temp/part files
    let mut candidate: Option<PathBuf> = None;
    for entry in fs::read_dir(dir).with_context(|| format!("listing {}", dir.display()))? {
        let p = entry?.path();

        // skip temporary/partial files
        if let Some(ext) = p.extension().and_then(|e| e.to_str())
            && (ext.eq_ignore_ascii_case("part") || ext.eq_ignore_ascii_case("tmp"))
        {
            continue;
        }

        if p.file_stem().and_then(|s| s.to_str()) == Some(stem) && p.is_file() {
            candidate = Some(p);
            break;
        }
    }
    Ok(candidate)
}

fn resolvers() -> [&'static dyn MediaResolver; 1] {
    static YT: YouTubeResolver = YouTubeResolver;
    [&YT]
}

/// resolve MediaInput: validate file exists if it's a file, or process URL through resolvers
pub fn resolve_media_input(media_input: MediaInput) -> Result<MediaInput> {
    match media_input {
        MediaInput::File(path) => {
            // validate that the file exists
            if !path.exists() {
                return Err(anyhow!("file does not exist: {}", path.display()));
            }
            Ok(MediaInput::File(path))
        }
        MediaInput::Url(url) => {
            // try parsing as url
            let parsed = Url::parse(&url).with_context(|| format!("invalid url: {url}"))?;

            // file:// scheme
            if parsed.scheme() == "file"
                && let Ok(path) = parsed.to_file_path()
            {
                if !path.exists() {
                    return Err(anyhow!("file does not exist: {}", path.display()));
                }
                return Ok(MediaInput::File(path));
            }

            // try resolvers
            for r in resolvers() {
                if r.matches(&parsed)
                    && let Some(mi) = r
                        .resolve(&url)
                        .with_context(|| format!("resolver {} failed", r.name()))?
                {
                    return Ok(mi);
                }
            }

            // fallback: treat as direct remote media url
            Ok(MediaInput::Url(url))
        }
    }
}

/// download/convert the media input to f32le pcm
pub fn download_pcm_from_source(
    media: &MediaInput,
    referer: Option<&str>,
    sample_rate_hz: Option<u32>,
    channels: Option<u8>,
) -> Result<PathBuf> {
    // defaults
    let sr: u32 = sample_rate_hz.unwrap_or(16_000);
    let ch: u8 = channels.unwrap_or(1);

    // validate numeric ranges
    if !(1..=8).contains(&ch) {
        return Err(anyhow!("channels must be in 1..=8, got {}", ch));
    }
    if !(8_000..=192_000).contains(&sr) {
        return Err(anyhow!(
            "sample_rate_hz must be in [8000, 192000], got {}",
            sr
        ));
    }

    // ensure cache dir
    let cache_dir = Path::new("./.cache/pcm");
    fs::create_dir_all(cache_dir).context("failed to create cache directory")?;

    // build stable hash from inputs
    let mut hasher = blake3::Hasher::new();
    match media {
        MediaInput::Url(u) => {
            hasher.update(u.as_bytes());
        }
        MediaInput::File(p) => {
            hasher.update(p.to_string_lossy().as_bytes());
        }
    }
    if let Some(r) = referer {
        hasher.update(r.as_bytes());
    }
    hasher.update(&sr.to_le_bytes());
    hasher.update(&[ch]);
    let hash = hasher.finalize().to_hex().to_string();

    // derive final and temp paths
    let filename = format!("{hash}.pcm");
    let output_path = cache_dir.join(filename);
    if output_path.exists() {
        return Ok(output_path);
    }
    let tmp_path = output_path.with_extension("tmp");

    // precompute strings
    let sr_s = sr.to_string();
    let ch_s = ch.to_string();
    let tmp_str = tmp_path.to_string_lossy().into_owned();

    // build ffmpeg args
    let mut args: Vec<String> = vec![
        "-nostdin".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
        "-i".into(),
    ];

    match media {
        MediaInput::File(path) => {
            args.push(path.to_string_lossy().into_owned());
        }
        MediaInput::Url(u) => {
            args.splice(
                0..0,
                vec!["-headers".into(), "User-Agent: Mozilla/5.0".into()],
            );
            let derived_referer = derive_referer(u);
            if let Some(r) = referer.or(derived_referer.as_deref()) {
                args.splice(0..0, vec!["-headers".into(), format!("Referer: {}", r)]);
            }
            args.push(u.clone());
        }
    }

    args.extend([
        "-ac".into(),
        ch_s,
        "-ar".into(),
        sr_s,
        "-f".into(),
        "f32le".into(),
        "-c:a".into(),
        "pcm_f32le".into(),
        tmp_str,
    ]);

    // run ffmpeg
    let status = Command::new("ffmpeg")
        .args(&args)
        .status()
        .context("failed to spawn ffmpeg")?;

    if !status.success() {
        return Err(anyhow!("ffmpeg exited with status {}", status));
    }

    fs::rename(&tmp_path, &output_path).with_context(|| {
        format!(
            "failed to move {} -> {}",
            tmp_path.display(),
            output_path.display()
        )
    })?;

    Ok(output_path)
}

/// read f32le pcm file into Vec<f32>
pub fn load_pcm_from_path(path: &Path) -> Result<Vec<f32>> {
    let buf =
        fs::read(path).with_context(|| format!("failed to read pcm file: {}", path.display()))?;

    let out = buf
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();

    Ok(out)
}

/// derive an origin-like referer from a media url, e.g. https://host[:port]/
fn derive_referer(media_url: &str) -> Option<String> {
    Url::parse(media_url).ok().and_then(|u| {
        let scheme = u.scheme();
        u.host_str().map(|h| {
            if let Some(port) = u.port() {
                format!("{scheme}://{h}:{port}/")
            } else {
                format!("{scheme}://{h}/")
            }
        })
    })
}
