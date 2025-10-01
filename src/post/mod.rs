mod chapters;
mod playlist;

use std::io::BufRead;
use std::path::PathBuf;

/// Common arguments to be used when calling `ffmpeg`.
const COMMON_ARGS: [&str; 5] = [
    "-y",
    "-nostats",
    "-hide_banner",
    "-loglevel",
    "repeat+level+info",
];

/// Forward `ffmpeg` logs to the [`log`]'s logger.
fn ffmpeg_log_forward(stderr: impl BufRead) {
    for line in stderr.lines() {
        if let Ok(msg) = line {
            if let Some((level, msg)) = ffmpeg_log_parse(&msg) {
                match level {
                    "info" => log::trace!("{}", msg),
                    "warning" => log::warn!("{}", msg),
                    "error" => log::error!("{}", msg),
                    _ => (),
                }
            }
        }
    }
}

/// Parse a `ffmpeg` log message.
///
/// Returns a tuple of `(level, message)`. The levels are: `error`, `warning`, `info`. Instead of
/// failing this function returns `None` if it can't parse the message.
fn ffmpeg_log_parse(message: &str) -> Option<(&str, &str)> {
    let mut level = None;
    let mut remainder = message;

    while let Some(start) = remainder.find('[') {
        if let Some(end) = remainder[start..].find(']') {
            let content = &remainder[start + 1..start + end];
            level = Some(content);
            remainder = &remainder[start + end + 1..];
        } else {
            break;
        }
    }

    level.map(|lbl| (lbl, remainder.trim_start()))
}

#[derive(Debug, thiserror::Error)]
pub enum PostError {
    #[error("cannot convert `{0}`: file not found")]
    FileNotFound(PathBuf, #[source] std::io::Error),
    #[error("cannot convert `{0}`: broken symlink (shouldn't be a symlink)")]
    BrokenPath(PathBuf),
    #[error("ffmpeg subprocess error")]
    SubprocessError(#[source] std::io::Error),
    #[error("ffmpeg returned with exit code `{0}`")]
    SubprocessFailed(String),
}
