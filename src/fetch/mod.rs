mod playlist;
pub use playlist::{Playlist, playlist};

use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use pythonize::depythonize;
use url::Url;

use crate::MDLError;
use crate::logger;

/// Python dispatcher to `yt_dlp --dump_single_json`.
///
/// In particular this function calls `ydl.extract_info(url, false)`, where 
/// `ydl` is a properly configured `YoutubeDL` instance.
fn fetch<T>(url: &Url) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let info = Python::attach(|py| -> crate::Result<T> {
        let info_dict = fetch_impl(py, url)?;
        let info = depythonize(&info_dict)?;
        Ok(info)
    })?;
    Ok(info)
}

fn fetch_impl<'py>(py: Python<'py>, url: &Url) -> PyResult<Bound<'py, PyAny>> {
    let logger = Bound::new(py, logger::MDLogger)?;
    let opts = PyDict::new(py);
    opts.set_item("dump_single_json", true)?;
    opts.set_item("extract_flat", false)?;
    opts.set_item("noprogress", true)?;
    opts.set_item("quiet", true)?;
    opts.set_item("simulate", true)?;
    opts.set_item("verbose", true)?;
    opts.set_item("color", "never")?;
    opts.set_item("format", "ba/b")?;
    let outtmpl = PyDict::new(py);
    outtmpl.set_item("default", "%(id)s")?;
    opts.set_item("outtmpl", outtmpl)?;
    opts.set_item("logger", logger)?;

    let yt_dlp = PyModule::import(py, "yt_dlp")?;
    let ydl = yt_dlp.getattr("YoutubeDL")?.call1((opts,))?;
    let info_dict = ydl.call_method1("extract_info", (url.as_str(), false))?;
    let info_dict = ydl.call_method1("sanitize_info", (info_dict,))?;
    Ok(info_dict)
}


/// Determine the URL which should be used when fetching playlists.
///
/// If `enable_sc` is `true` Soundcloud URLs will be accepted and checked, passing a Soundcloud URL
/// is an error otherwise.
///
/// This function will fail if `url` doesn't meet the requirements:
/// - for YouTube the URL must be of the form:
///   * `https://www.youtube.com/playlist?list=<id>`
///   * `https://www.youtube.com/watch?v=<vid>&list=<id>`
/// - for Soundcloud the URL must be of the form:
///   * `https://soundcloud.com/<author>/sets/<id>` or in other words it has to contain `sets` in its path.
fn get_playlist_url(url: &Url, enable_sc: bool) -> crate::Result<Url> {
    match (url.host_str(), enable_sc) {
        (Some("www.youtube.com"), _) => {
            let id = url
                .query_pairs()
                .find_map(|(k, v)| if k == "list" { Some(v) } else { None })
                .ok_or_else(|| GetUrlError::NotPlaylist(url.to_string()))?;

            let mut url = url.clone();
            url.set_path("playlist");
            url.set_query(Some(&format!("list={}", id)));

            Ok(url)
        }
        (Some("soundcloud.com"), true) => {
            url.path_segments()
                .ok_or_else(|| GetUrlError::Unsound(url.to_string()))?
                .find(|s| *s == "sets")
                .ok_or_else(|| GetUrlError::NotPlaylist(url.to_string()))?;

            Ok(url.clone())
        }
        _ => Err(GetUrlError::Unsupported(url.to_string()).into()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetUrlError {
    #[error("`{0}` is not a playlist URL")]
    NotPlaylist(String),
    #[error("`{0}` is an unsound (we don't know what to do with it) URL")]
    Unsound(String),
    #[error("fetching `{0}` is not supported")]
    Unsupported(String),
}
