use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use url::Url;

/// Structure representing a playlist specfile.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    #[serde(rename = "playlist")]
    header: PlaylistHeader,
    #[serde(rename = "track")]
    tracks: Vec<PlaylistTrack>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PlaylistHeader {
    id: String,
    title: String,
    release_year: String,
    album_artists: Vec<String>,
    genre: String,
    track_total: u32,
    cover: Url,
}

/// A playlist track, part of the playlist spec.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PlaylistTrack {
    url: Url,
    title: String,
    artists: Vec<String>,
    track_num: u32,
}

impl Playlist {
    /// Read a playlist spec from `path`.
    pub fn read_from(path: impl AsRef<Path>) -> crate::Result<Self> {
        log::debug!("Reading playlist specfile from {}", path.as_ref().display());
        read_from_impl(path)
    }
}

/// Structure representing a chapters specfile.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Chapters {
    #[serde(rename = "chapters")]
    header: ChaptersHeader,
    #[serde(rename = "track")]
    tracks: Vec<ChapterTrack>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ChaptersHeader {
    id: String,
    url: Url,
    title: String,
    release_year: String,
    album_artists: Vec<String>,
    genre: String,
    cover: Url,
}

/// A chapter track, part of chapters spec.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ChapterTrack {
    title: String,
    artists: Vec<String>,
    start_time: f32,
    end_time: f32,
}

impl Chapters {
    /// Read chapters spec from `path`.
    pub fn read_from(path: impl AsRef<Path>) -> crate::Result<Self> {
        log::debug!("Reading chapters specfile from {}", path.as_ref().display());
        read_from_impl(path)
    }
}

/// An enum representing a spec, either [`Playlist`] or [`Chapters`].
///
/// Look at [`Playlist::read_from()`] or [`Chapters::read_from()`] if the type of the spec is
/// already known.
pub enum Spec {
    Playlist(Playlist),
    Chapters(Chapters),
}

impl Spec {
    /// Read the specfile at `path` and determine its type.
    ///
    /// This function will fail if the specfile is valid TOML, but not a known specfile format.
    pub fn read_from(path: impl AsRef<Path>) -> crate::Result<Self> {
        log::debug!("Reading specfile from {}", path.as_ref().display());
        let spec: toml::Table = read_from_impl(path.as_ref())?;
        log::debug!("Trying to guess the specfile format");
        if spec.contains_key("playlist") {
            log::debug!("Determined the specfile to be a playlist");
            match spec.try_into() {
                Ok(s) => Ok(Self::Playlist(s)),
                Err(e) => {
                    log::error!("Deserializing {} failed:\n{}", path.as_ref().display(), e);
                    Err(SpecError::Deserialization(path.as_ref().to_path_buf(), e).into())
                }
            }
        } else if spec.contains_key("chapters") {
            log::debug!("Determined the specfile to be chapters");
            match spec.try_into() {
                Ok(s) => Ok(Self::Chapters(s)),
                Err(e) => {
                    log::error!("Deserializing {} failed:\n{}", path.as_ref().display(), e);
                    Err(SpecError::Deserialization(path.as_ref().to_path_buf(), e).into())
                }
            }
        } else {
            log::error!("Unknown specfile format of {}", path.as_ref().display());
            Err(SpecError::UnknownFormat(path.as_ref().to_path_buf()).into())
        }
    }
}

/// Specfile related errors - reading, writing, de/serializing
#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("serializing `{0}` failed")]
    Serialization(String, #[source] toml::ser::Error),
    #[error("deserializing `{0}` failed")]
    Deserialization(PathBuf, #[source] toml::de::Error),
    #[error("IO error when {0} the specfile at `{1}`")]
    IoError(&'static str, PathBuf, #[source] io::Error),
    #[error("cannot read `{0}`: unknown specfile format")]
    UnknownFormat(PathBuf),
}

fn read_from_impl<T, P>(path: P) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<Path>,
{
    let spec_bytes = match fs::read(path.as_ref()) {
        Ok(b) => b,
        Err(e) => {
            log::error!("Reading {} failed: {}", path.as_ref().display(), e);
            return Err(SpecError::IoError("reading", path.as_ref().to_path_buf(), e).into());
        }
    };

    let spec = match toml::from_slice(&spec_bytes) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Deserializing {} failed:\n{}", path.as_ref().display(), e);
            return Err(SpecError::Deserialization(path.as_ref().to_path_buf(), e).into());
        }
    };
    Ok(spec)
}
