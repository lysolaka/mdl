use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use url::Url;

use crate::specfile::{Playlist, PlaylistTrack};

use super::DownloadError;

impl Playlist {
    /// Download all tracks in the playlist.
    ///
    /// If `outdir` is `None`, put the downloaded tracks in the `./<playlist-id>/` directory, when
    /// `outdir` is `Some(path)` the downloaded will be put to `<path>/<playlist-id>/`.
    pub fn download(&self, outdir: Option<impl AsRef<Path>>) -> crate::Result<()> {
        let outdir = match outdir {
            Some(dir) => dir.as_ref().join(self.id()),
            None => PathBuf::from(self.id()),
        };

        log::debug!("Creating output directory {}", outdir.display());
        if let Err(e) = fs::create_dir_all(&outdir) {
            log::error!(
                "Cannot create the output directory {}: {}",
                outdir.display(),
                e
            );
            return Err(DownloadError::IoError("creating the output directory", e).into());
        }

        let mut is_complete = true;
        let len = self.tracks().len();
        for (i, t) in self
            .tracks()
            .enumerate()
            .filter(|(i, t)| filter_track_ok(i, t, len))
        {
            let outpath = outdir.join(t.id());
            if t.url().scheme() == "file" {
                log::info!(
                    "({}/{}) Copying track: {} ({})",
                    i + 1,
                    len,
                    t.title(),
                    t.url()
                );
                log::debug!("({}/{}) Copying to: {}", i + 1, len, outpath.display());
                let file = match t.url().to_file_path() {
                    Ok(p) => p,
                    Err(_) => {
                        log::error!(
                            "({}/{}) Copying {} ({}) failed: cannot determine the input file path",
                            i + 1,
                            len,
                            t.title(),
                            t.url()
                        );
                        is_complete = false;
                        continue;
                    }
                };
                if let Err(e) = fs::copy(file, outpath) {
                    log::error!(
                        "({}/{}) Copying {} ({}) failed: {}",
                        i + 1,
                        len,
                        t.title(),
                        t.url(),
                        e
                    );
                    is_complete = false;
                }
            } else {
                log::info!(
                    "({}/{}) Downloading track: {} ({})",
                    i + 1,
                    len,
                    t.title(),
                    t.url()
                );
                log::debug!("({}/{}) Downloading to: {}", i + 1, len, outpath.display());
                if let Err(e) = super::download(t.url(), outpath) {
                    log::error!(
                        "({}/{}) Downloading {} ({}) failed: {}",
                        i + 1,
                        len,
                        t.title(),
                        t.url(),
                        e
                    );
                    is_complete = false;
                    if let Some(source) = e.source() {
                        log::error!("{}", source);
                    }
                }
            }
        }
        log::info!(
            "Downloaded playlist {} ({}) to {}",
            self.title(),
            self.id(),
            outdir.display()
        );
        if is_complete {
            Ok(())
        } else {
            Err(DownloadError::Incomplete.into())
        }
    }
}

fn filter_track_ok(i: &usize, t: &PlaylistTrack, len: usize) -> bool {
    if t.is_url_ok() {
        true
    } else {
        log::warn!(
            "({}/{}) Skipping the download of {} ({}): the URL is not a file or former output of fetch",
            i + 1,
            len,
            t.title(),
            t.url()
        );
        false
    }
}
