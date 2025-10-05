use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::specfile::{Chapters, Format};

use super::TagError;

impl Chapters {
    /// Tag the chapter album's converted files (searched for in `<indir>/<self.id>` or `<self.id>` if
    /// `indir` is `None`).
    ///
    /// Remember to postprocess the album first (see [`Chapters::postprocess()`]).
    ///
    /// The output files will be located at `<outdir>/<self.title>` or `<self.title>` if `outdir`
    /// is `None`.
    pub fn tag<P>(&self, target: Format, indir: Option<P>, outdir: Option<P>) -> crate::Result<()>
    where
        P: AsRef<Path>,
    {
        log::info!("Tagging {}", self.title());
        let indir = match indir {
            Some(d) => d.as_ref().join(self.id()),
            None => PathBuf::from(self.id()),
        };
        log::debug!("Postprocessed files expected to be in {}", indir.display());
        match fs::exists(&indir) {
            Ok(true) => (),
            _ => {
                log::error!("Cannot find the input files to tag {}", self.title());
                return Err(TagError::NoIndir(self.title().to_string(), indir).into());
            }
        }

        let outdir = match outdir {
            Some(d) => d.as_ref().join(self.title()),
            None => PathBuf::from(self.title()),
        };

        log::debug!("Creating output directory {}", outdir.display());
        if let Err(e) = fs::create_dir_all(&outdir) {
            log::error!(
                "Cannot create the output directory {}: {}",
                outdir.display(),
                e
            );
            return Err(TagError::IoError("creating the output directory", e).into());
        }

        let (cover, mime) = match super::get_cover(self.cover_url()) {
            Ok(cm) => cm,
            Err(e) => {
                log::error!("Cannot get the cover art for {}", self.title());
                return Err(e);
            }
        };

        let file_id = self.file_id();
        let len = self.tracks().len();
        for (i, track) in self.tracks().enumerate() {
            log::info!("({}/{}) Tagging {}", i + 1, len, track.title());
            let tag = track.tag((i as u32 + 1, len as u32), &self);
            let outpath = outdir.join(format!("{:02}. {}.{}", tag.track, tag.title, target.ext()));
            let inpath = indir.join(format!("{}_{}.{}", i + 1, file_id, target.ext()));
            log::debug!("Copying {} to \"{}\"", inpath.display(), outpath.display());
            if let Err(e) = fs::copy(&inpath, &outpath) {
                log::error!("Copying {} failed: {}", inpath.display(), e);
                log::warn!("Skipping tagging {}", tag.title);
                continue;
            }
            log::debug!("Tagging \"{}\"", outpath.display());
            if let Err(e) = tag.write_tag(&outpath, target, cover.clone(), mime) {
                log::error!("Tagging \"{}\" failed: {}", outpath.display(), e);
                let mut source = e.source();
                if source.is_some() {
                    log::error!("Caused by:");
                    let mut i = 0;

                    while let Some(err) = source {
                        log::error!("{}: {}", i, err);
                        source = err.source();
                        i += 1;
                    }
                }
            }
        }
        log::info!(
            "Finished tagging {}. The tagged files can be found in {}",
            self.title(),
            outdir.display()
        );
        Ok(())
    }
}
