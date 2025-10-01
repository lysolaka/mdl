use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::specfile::{Format, Playlist};

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
        todo!()
    }
}
