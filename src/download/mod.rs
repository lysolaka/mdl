mod chapters;
mod playlist;

use std::path::Path;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use url::Url;

use crate::logger;

fn download(url: &Url, outpath: impl AsRef<Path>) -> Result<(), DownloadError> {
    Python::attach(|py| -> PyResult<()> {
        let logger = Bound::new(py, logger::MDLogger)?;
        let opts = PyDict::new(py);

        opts.set_item("extract_flat", false)?;

        let extractor_args = PyDict::new(py);
        let youtube = PyDict::new(py);
        youtube.set_item("player_client", ["mweb"])?;
        extractor_args.set_item("youtube", youtube)?;
        opts.set_item("extractor_args", extractor_args)?;

        opts.set_item("noprogress", true)?;
        opts.set_item("verbose", true)?;
        opts.set_item("color", "never")?;
        opts.set_item("format", "ba/b")?;

        let outtmpl = PyDict::new(py);
        outtmpl.set_item("default", outpath.as_ref().to_string_lossy())?;
        opts.set_item("outtmpl", outtmpl)?;

        opts.set_item("overwrites", true)?;
        opts.set_item("logger", logger)?;

        let yt_dlp = PyModule::import(py, "yt_dlp")?;
        let ydl = yt_dlp.getattr("YoutubeDL")?.call1((opts,))?;
        ydl.getattr("download")?.call1((vec![url.as_str()],))?;
        Ok(())
    })?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("unhandled Python exception")]
    Python(#[from] pyo3::PyErr),
    #[error("IO error when {0}")]
    IoError(&'static str, #[source] std::io::Error),
    #[error("the URL `{0}` is not a former output of fetch")]
    InvalidURL(Url),
}
