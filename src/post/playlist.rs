use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::specfile::{Format, Playlist, PlaylistTrack};

use super::PostError;

impl Playlist {
    /// Convert all playlist files to `format`.
    ///
    /// Call this after downloading the playlist. This function expects the files to be
    /// untouched after the download. Thus `dir` should be set to the same thing as `outdir` in
    /// [`Playlist::download()`].
    ///
    /// If `keep` is `true`, the old, downloaded files will be kept.
    pub fn postprocess(
        &self,
        target: Format,
        keep: bool,
        dir: Option<impl AsRef<Path>>,
    ) -> crate::Result<()> {
        let dir = match dir {
            Some(dir) => dir.as_ref().join(self.id()),
            None => PathBuf::from(self.id()),
        };

        log::info!("Converting {} ({}) to {}", self.title(), self.id(), target);
        let len = self.tracks().len();
        for (i, track) in self.tracks().enumerate() {
            let file_path = dir.join(track.id());
            log::info!("({}/{}) Converting {}", i + 1, len, track.title());
            if let Err(e) = check_exists(&file_path) {
                log::error!("{}", e);
                if let Some(s) = e.source() {
                    log::error!("Caused by: {}", s);
                }
                log::info!("({}/{}) Skipping {}", i + 1, len, track.title());
                continue;
            }

            let outfile = dir.join(format!("{}.{}", track.id(), target.ext()));
            if let Err(e) = track.convert(&file_path, &outfile, target) {
                log::error!(
                    "({}/{}) Converting {} failed: {}",
                    i + 1,
                    len,
                    track.title(),
                    e
                );
                if let Some(s) = e.source() {
                    log::error!("Caused by: {}", s);
                }
                log::info!("({}/{}) Skipping {}", i + 1, len, track.title());
                continue;
            }
            log::debug!("Saving to {}", outfile.display());

            if !keep {
                log::debug!("Removing the original file (`keep` is false)");
                if let Err(e) = fs::remove_file(&file_path) {
                    log::warn!("Failed to remove {}: {}", file_path.display(), e);
                }
            }
        }

        log::info!("Finished converting {}", self.title());
        Ok(())
    }
}

impl PlaylistTrack {
    fn convert<P1, P2>(&self, file: P1, outfile: P2, target: Format) -> crate::Result<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let mut child = Command::new("ffmpeg")
            .env("AV_LOG_FORCE_NOCOLOR", "1")
            .args(super::COMMON_ARGS)
            .arg("-i")
            .arg(file.as_ref())
            .arg("-ignore_unknown")
            .arg("-map")
            .arg("0:a")
            .arg("-map_metadata")
            .arg("-1")
            .arg("-c:a")
            .arg(target.codec())
            .arg("-q:a")
            .arg(target.quality())
            .arg(outfile.as_ref())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PostError::SubprocessError(e))?;

        let logger = child.stderr.take().map_or_else(
            || {
                log::warn!("Failed to capture ffmpeg's stderr. No logs are available");
                None
            },
            |stderr| {
                Some(std::thread::spawn(move || {
                    super::ffmpeg_log_forward(BufReader::new(stderr))
                }))
            },
        );

        let status = child.wait().map_err(|e| PostError::SubprocessError(e))?;
        if let Some(logger) = logger {
            let _ = logger.join();
        }

        match status.success() {
            true => Ok(()),
            false => Err(PostError::SubprocessFailed(status.to_string()).into()),
        }
    }
}

fn check_exists(path: impl AsRef<Path>) -> crate::Result<()> {
    match fs::exists(path.as_ref()) {
        Ok(true) => Ok(()),
        Ok(false) => Err(PostError::BrokenPath(path.as_ref().to_path_buf()).into()),
        Err(e) => Err(PostError::FileNotFound(path.as_ref().to_path_buf(), e).into()),
    }
}
