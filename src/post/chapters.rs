use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::specfile::{ChapterTrack, Chapters, Format};

use super::PostError;

impl Chapters {
    /// Convert to `format` and split the downloaded file into tracks.
    ///
    /// Call this after downloading the chapter file. This function expects the file to be
    /// untouched after the download. Thus `dir` should be set to the same thing as `outdir` in
    /// [`Chapters::download()`].
    ///
    /// If `keep` is `true`, the old, downloaded file will be kept.
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

        let file_id = self.file_id();
        let file_path = dir.join(&file_id);
        // check if it exists
        match fs::exists(&file_path) {
            Ok(true) => log::info!(
                "Splitting and converting {} ({}) to {}",
                self.title(),
                file_path.display(),
                target
            ),
            Ok(false) => return Err(PostError::BrokenPath(file_path).into()),
            Err(e) => return Err(PostError::FileNotFound(file_path, e).into()),
        }

        let len = self.tracks().len();
        for (i, track) in self.tracks().enumerate() {
            let outfile = dir.join(format!("{}_{}.{}", i + 1, &file_id, target.ext()));
            log::info!("({}/{}) Splitting {}", i + 1, len, track.title());
            if let Err(e) = track.split_from(&file_path, &outfile, target) {
                log::error!(
                    "({}/{}) Splitting {} failed: {}",
                    i + 1,
                    len,
                    track.title(),
                    e
                );
                if let Some(s) = e.source() {
                    log::error!("Caused by: {}", s);
                }
                return Err(e);
            }
            log::debug!("Saving to {}", outfile.display());
        }
        log::info!("Finished splitting and converting {}", self.title());
        if !keep {
            log::debug!("Removing the original file (`keep` is false)");
            if let Err(e) = fs::remove_file(&file_path) {
                log::warn!("Failed to remove {}: {}", file_path.display(), e);
            }
        }
        Ok(())
    }
}

impl ChapterTrack {
    fn split_from<P1, P2>(&self, file: P1, outfile: P2, target: Format) -> crate::Result<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let (start, end) = self.range();
        log::debug!("Split range: {}s : {}s", start, end);
        let mut child = Command::new("ffmpeg")
            .env("AV_LOG_FORCE_NOCOLOR", "1")
            .args(super::COMMON_ARGS)
            .arg("-ss")
            .arg(format!("{start}"))
            .arg("-to")
            .arg(format!("{end}"))
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
            logger.join();
        }

        match status.success() {
            true => Ok(()),
            false => Err(PostError::SubprocessFailed(status.to_string()).into()),
        }
    }
}
