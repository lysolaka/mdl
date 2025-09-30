use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use url::Url;

use crate::specfile::Chapters;

use super::DownloadError;

impl Chapters {
    /// Download the file containing the chapters.
    ///
    /// If `outdir` is `None`, put the downloaded file in the `./<chapters-id>/` directory, when
    /// `outdir` is `Some(path)` the downloaded file will be put to `<path>/<chapters-id>/`.
    pub fn download(&self, outdir: Option<impl AsRef<Path>>) -> crate::Result<()> {
        // check the URL first
        if !self.is_url_ok() {
            let e = DownloadError::InvalidURL(self.url().clone());
            log::error!("Cannot download {} ({}): {}", self.title(), self.url(), e);
            return Err(e.into());
        }

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

        let outpath = outdir.join(self.file_id());
        log::info!("Downloading {} ({})", self.title(), self.url());
        log::debug!("Saving to {}", outpath.display());
        if let Err(e) = super::download(self.url(), outpath) {
            log::error!(
                "Downloading {} ({}) failed: {}",
                self.title(),
                self.url(),
                e
            );
            if let Some(e) = e.source() {
                log::error!("{}", e);
            }
            return Err(e.into());
        }
        Ok(())
    }
}
