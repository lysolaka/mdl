mod playlist;
pub use playlist::{Playlist, playlist};

mod chapters;
pub use chapters::{Chapters, chapters, chapters_recursive};

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use pythonize::depythonize;
use url::Url;

use crate::logger;
use crate::specfile::SpecError;

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
        let info = match depythonize(&info_dict) {
            Ok(info) => info,
            Err(e) => {
                log::error!("{}", e);
                return Err(e.into());
            }
        };
        Ok(info)
    })?;
    Ok(info)
}

fn fetch_impl<'py>(py: Python<'py>, url: &Url) -> PyResult<Bound<'py, PyAny>> {
    let logger = Bound::new(py, logger::MDLogger)?;
    let opts = PyDict::new(py);
    opts.set_item("dump_single_json", true)?;
    opts.set_item("extract_flat", false)?;

    let extractor_args = PyDict::new(py);
    let youtube = PyDict::new(py);
    youtube.set_item("player_client", ["mweb"])?;
    extractor_args.set_item("youtube", youtube)?;
    opts.set_item("extractor_args", extractor_args)?;

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

fn write_to_impl<T, P>(s: &T, id: &str, destdir: Option<P>) -> crate::Result<()>
where
    T: serde::Serialize,
    P: AsRef<Path>,
{
    let file_name = format!("{}.toml", id);
    let path = match destdir {
        Some(dir) => match fs::create_dir_all(dir.as_ref()) {
            Ok(_) => dir.as_ref().join(file_name),
            Err(e) => {
                log::error!("Cannot create directory {}: {}", dir.as_ref().display(), e);
                return Err(SpecError::IoError("creating", dir.as_ref().to_path_buf(), e).into());
            }
        },
        None => PathBuf::from(file_name),
    };

    log::info!("Saving specfile {} to {}", id, path.display());
    log::debug!("Serializing {} to TOML", id);
    let spec_str = match toml::to_string(s) {
        Ok(s) => s,
        Err(e) => {
            log::error!("TOML serialization failed: {}", e);
            return Err(SpecError::Serialization(id.to_string(), e).into());
        }
    };

    let mut file = match fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Cannot create {}: {}", path.display(), e);
            return Err(SpecError::IoError("creating", path, e).into());
        }
    };
    let write: io::Result<()> = {
        file.write_all(spec_str.as_bytes())?;
        file.flush()?;
        Ok(())
    };

    if let Err(e) = write {
        log::error!("Error writing {}: {}", path.display(), e);
        return Err(SpecError::IoError("writing", path, e).into());
    }
    log::debug!("Successfully written {} to {}", id, path.display());
    Ok(())
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
fn get_playlist_url(url: &Url, enable_sc: bool) -> Result<Url, GetUrlError> {
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

/// Extract the video URL from a possible video and playlist URL.
///
/// This function returns the parsed video URL. Only YouTube links to videos are allowed.
fn get_video_url(url: &Url) -> Result<Url, GetUrlError> {
    match url.host_str() {
        Some("www.youtube.com") => {
            let vid = url
                .query_pairs()
                .find_map(|(k, v)| if k == "v" { Some(v) } else { None })
                .ok_or_else(|| GetUrlError::NotVideo(url.to_string()))?;

            let mut url = url.clone();
            url.set_path("watch");
            url.set_query(Some(&format!("v={}", vid)));

            Ok(url)
        }
        _ => Err(GetUrlError::Unsupported(url.to_string()).into()),
    }
}

/// Additional URL parsing errors.
#[derive(Debug, thiserror::Error)]
pub enum GetUrlError {
    #[error("`{0}` is not a playlist URL")]
    NotPlaylist(String),
    #[error("`{0}` is not a video URL")]
    NotVideo(String),
    #[error("`{0}` is an unsound (we don't know what to do with it) URL")]
    Unsound(String),
    #[error("fetching `{0}` is not supported")]
    Unsupported(String),
}

#[cfg(test)]
mod tests {
    use super::get_playlist_url;
    use super::get_video_url;
    use url::Url;

    #[test]
    fn youtube_playlist_url() -> crate::Result<()> {
        let u =
            Url::parse("https://www.youtube.com/playlist?list=PLx6F6orIEZdmbMQyQKYXq2R0XLgYk1Lpw")?;
        let url = get_playlist_url(&u, false)?;
        assert_eq!(url, u);

        let u = Url::parse(
            "https://www.youtube.com/watch?v=rzurrERdmpI&list=PLx6F6orIEZdmbMQyQKYXq2R0XLgYk1Lpw",
        )?;
        let url = get_playlist_url(&u, false)?;
        assert_eq!(
            url,
            Url::parse("https://www.youtube.com/playlist?list=PLx6F6orIEZdmbMQyQKYXq2R0XLgYk1Lpw")?
        );

        let u = Url::parse("https://www.youtube.com/watch?v=rzurrERdmpI")?;
        assert!(get_playlist_url(&u, false).is_err());
        Ok(())
    }

    #[test]
    fn soundcloud_playlist_url() -> crate::Result<()> {
        let u =
            Url::parse("https://soundcloud.com/alcomindz/sets/koldi-kolins-dawaj-mixtape-2018")?;
        let url = get_playlist_url(&u, true)?;
        assert_eq!(url, u);

        let u =
            Url::parse("https://soundcloud.com/alcomindz/sets/koldi-kolins-dawaj-mixtape-2018")?;
        assert!(get_playlist_url(&u, false).is_err());

        let u = Url::parse("https://soundcloud.com/alcomindz/albums")?;
        assert!(get_playlist_url(&u, true).is_err());
        Ok(())
    }

    #[test]
    fn unsound_playlist_url() -> crate::Result<()> {
        let u = Url::parse("https://www.nottube.ru/watch?v=rzurrERdmpI")?;
        assert!(get_playlist_url(&u, true).is_err());

        let u = Url::parse("https://soundcloud.com")?;
        assert!(get_playlist_url(&u, true).is_err());
        Ok(())
    }

    #[test]
    fn youtube_video_url() -> crate::Result<()> {
        let u = Url::parse(
            "https://www.youtube.com/watch?v=vnd35SLG4Yc&list=PLqWr7dyJNgLKjMUfZ7mnuPFLxX813sm8K",
        )?;
        let url = get_video_url(&u)?;
        assert_eq!(
            url,
            Url::parse("https://www.youtube.com/watch?v=vnd35SLG4Yc")?
        );

        let u = Url::parse("https://www.youtube.com/watch?v=vnd35SLG4Yc")?;
        let url = get_video_url(&u)?;
        assert_eq!(url, u);

        let u =
            Url::parse("https://www.youtube.com/playlist?list=PLqWr7dyJNgLK45KSPhhti4FcWLhEWlegt")?;
        assert!(get_video_url(&u).is_err());
        Ok(())
    }

    #[test]
    fn unsound_video_url() -> crate::Result<()> {
        let u =
            Url::parse("https://soundcloud.com/alcomindz/sets/koldi-kolins-dawaj-mixtape-2018")?;
        assert!(get_video_url(&u).is_err());

        let u = Url::parse("https://docs.rs")?;
        assert!(get_video_url(&u).is_err());
        Ok(())
    }
}
