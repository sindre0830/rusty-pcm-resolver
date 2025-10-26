use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use url::Url;

pub mod domain;
pub mod resolver;

static INIT: Once = Once::new();

pub fn ensure_init() {
    INIT.call_once(resolver::builtin::register_builtins);
}

/// user-configurable options for conversion
#[derive(Clone, Debug)]
pub struct Options {
    pub referer: Option<String>,
    pub sample_rate_hz: u32,
    pub channels: u8,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            referer: None,
            sample_rate_hz: 16_000,
            channels: 1,
        }
    }
}

impl Options {
    /// set target sample rate
    pub fn sample_rate(mut self, hz: u32) -> Self {
        self.sample_rate_hz = hz;
        self
    }

    /// set number of audio channels
    pub fn channels(mut self, ch: u8) -> Self {
        self.channels = ch;
        self
    }

    /// set explicit referer header
    pub fn referer(mut self, r: Option<impl Into<String>>) -> Self {
        self.referer = r.map(Into::into);
        self
    }
}

/// internal state machine for the fluent api
enum State {
    Init,
    Resolved(domain::MediaInput),
    Converted(PathBuf),
}

/// fluent pipeline for resolving -> converting -> loading
pub struct PcmResolver {
    opts: Options,
    state: State,
}

impl PcmResolver {
    /// create a new resolver with options
    pub fn new(opts: Options) -> Self {
        Self {
            opts,
            state: State::Init,
        }
    }

    /// resolve the input (local file, file://, youtube, etc.) into a concrete media source
    pub fn resolve_media(mut self, input: domain::MediaInput) -> Result<Self> {
        // initialize builtin resolvers on first use
        ensure_init();

        let resolved = match input {
            domain::MediaInput::File(path) => {
                // validate that the file exists
                if !path.exists() {
                    return Err(anyhow!("file does not exist: {}", path.display()));
                }
                domain::MediaInput::File(path)
            }
            domain::MediaInput::Url(url) => {
                // parse as url
                let parsed = Url::parse(&url).with_context(|| format!("invalid url: {url}"))?;

                // handle local files
                if parsed.scheme() == "file" {
                    let path = parsed
                        .to_file_path()
                        .map_err(|_| anyhow!("invalid file url: {url}"))?;
                    if !path.exists() {
                        return Err(anyhow!("file does not exist: {}", path.display()));
                    }
                    domain::MediaInput::File(path)
                } else {
                    // try custom resolvers (builtin + user-registered)
                    for r in resolver::resolvers() {
                        if r.matches(&parsed)
                            && let Some(mi) = r
                                .resolve(&url)
                                .with_context(|| format!("resolver {} failed", r.name()))?
                        {
                            // short-circuit on first successful resolver
                            self.state = State::Resolved(mi);
                            return Ok(self);
                        }
                    }
                    // fallback: treat as direct remote media url
                    domain::MediaInput::Url(url)
                }
            }
        };

        self.state = State::Resolved(resolved);
        Ok(self)
    }

    /// run ffmpeg to convert to f32le pcm and cache the result; yields a path handle
    pub fn convert_to_pcm(mut self) -> Result<Self> {
        let media = match &self.state {
            State::Resolved(m) => m,
            _ => return Err(anyhow!("convert_to_pcm() requires resolve_media() first")),
        };

        // defaults
        let sr = self.opts.sample_rate_hz;
        let ch = self.opts.channels;

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
            domain::MediaInput::Url(u) => hasher.update(u.as_bytes()),
            domain::MediaInput::File(p) => hasher.update(p.to_string_lossy().as_bytes()),
        };
        if let Some(r) = &self.opts.referer {
            hasher.update(r.as_bytes());
        }
        hasher.update(&sr.to_le_bytes());
        hasher.update(&[ch]);
        let hash = hasher.finalize().to_hex().to_string();

        // derive final and temp paths
        let filename = format!("{hash}.pcm");
        let output_path = cache_dir.join(filename);
        if output_path.exists() {
            self.state = State::Converted(output_path);
            return Ok(self);
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
            domain::MediaInput::File(path) => {
                args.push(path.to_string_lossy().into_owned());
            }
            domain::MediaInput::Url(u) => {
                // add headers up front
                args.splice(
                    0..0,
                    vec!["-headers".into(), "User-Agent: Mozilla/5.0".into()],
                );
                let derived = derive_referer(u);
                if let Some(r) = self.opts.referer.as_deref().or(derived.as_deref()) {
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

        self.state = State::Converted(output_path);
        Ok(self)
    }

    /// load samples from the converted pcm file
    pub fn load(&self) -> Result<Vec<f32>> {
        let p = match &self.state {
            State::Converted(p) => p,
            _ => return Err(anyhow!("load() requires convert_to_pcm() first")),
        };

        let buf =
            fs::read(p).with_context(|| format!("failed to read pcm file: {}", p.display()))?;
        let out = buf
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        Ok(out)
    }

    /// get a borrowed path to the converted pcm (after convert_to_pcm)
    pub fn path(&self) -> Option<&Path> {
        match &self.state {
            State::Converted(p) => Some(p.as_path()),
            _ => None,
        }
    }

    /// consume and return the converted pcm path
    pub fn into_path(self) -> Result<PathBuf> {
        match self.state {
            State::Converted(p) => Ok(p),
            _ => Err(anyhow!("into_path() requires convert_to_pcm() first")),
        }
    }
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
