use std::fs;
use std::hash::Hasher;
use std::io;
use std::path::{Path, PathBuf};

use rustc_hash::FxHasher;
use url::Url;

use crate::tag::Tag;

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

    /// Returns the playlist's "user" ID.
    pub fn id(&self) -> &str {
        &self.header.id
    }

    /// Returns the playlist's title.
    pub fn title(&self) -> &str {
        &self.header.title
    }

    /// Returns the cover URL.
    pub fn cover_url(&self) -> &Url {
        &self.header.cover
    }

    /// An iterator over the playlist's tracks.
    pub fn tracks(&self) -> std::slice::Iter<'_, PlaylistTrack> {
        self.tracks.iter()
    }
}

impl PlaylistTrack {
    /// Returns the track's URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the track's title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The [`PlaylistTrack`]'s id, used as a resulting filename for downloads and copies.
    ///
    /// It is implemented as a non-cryptografic hash of the URL hashed using [`rustc_hash::FxHasher`].
    ///
    /// This function DOES NOT validate the URL.
    pub fn id(&self) -> String {
        let mut hasher = FxHasher::with_seed(0x243f6a8885a308d3);
        hasher.write(self.url.as_str().as_bytes());
        let h = hasher.finish();
        format!("{h:016x}")
    }

    /// Check if the URL is a former output of fetch.
    ///
    /// If the URL has scheme `file`, this function will return `true`.
    pub fn is_url_ok(&self) -> bool {
        if self.url.scheme() == "file" {
            return true;
        }

        match self.url.host_str() {
            Some("www.youtube.com") => {
                let is_video = self.url.query_pairs().find(|(k, _)| k == "v").is_some();
                let is_single = self.url.query_pairs().count() == 1;
                is_video && is_single
            }
            Some("api-v2.soundcloud.com") => self.url.path_segments().map_or(false, |mut ps| {
                let tracks = ps.next().map_or(false, |s| s == "tracks");
                let id = ps.next().map_or(false, |id| id.parse::<u64>().is_ok());
                let nothing = ps.next().is_none();
                tracks && id && nothing
            }),
            _ => false,
        }
    }

    /// Construct a [`Tag`] representing the [`PlaylistTrack`].
    ///
    /// `parent` is the playlist album spec, which the track is a part of.
    pub fn tag(&self, parent: &Playlist) -> Tag {
        let year = parent.header.release_year.chars().take(4).collect();
        Tag {
            title: self.title.clone(),
            album_title: parent.header.title.clone(),
            track: self.track_num,
            track_total: parent.header.track_total,
            genre: parent.header.genre.clone(),
            artists: self.artists.join(", "),
            album_artists: parent.header.album_artists.join(", "),
            release_year: year,
        }
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

    /// Returns the chapter album's "user" ID.
    pub fn id(&self) -> &str {
        &self.header.id
    }

    /// Return the chapters' source URL.
    pub fn url(&self) -> &Url {
        &self.header.url
    }

    /// Returns the cover URL.
    pub fn cover_url(&self) -> &Url {
        &self.header.cover
    }

    /// The [`Chapters`] file ID used for uniquely naming files.
    ///
    /// It is implemented as a non-cryptografic hash of the URL hashed using [`rustc_hash::FxHasher`].
    ///
    /// This function DOES NOT validate the URL.
    pub fn file_id(&self) -> String {
        let mut hasher = FxHasher::with_seed(0x24af6a1405a3c813);
        hasher.write(self.header.url.as_str().as_bytes());
        let h = hasher.finish();
        format!("{h:016x}")
    }

    /// Checks if the URL is a former output of fetch.
    pub fn is_url_ok(&self) -> bool {
        match self.header.url.host_str() {
            Some("www.youtube.com") => {
                let is_video = self
                    .header
                    .url
                    .query_pairs()
                    .find(|(k, _)| k == "v")
                    .is_some();
                let is_single = self.header.url.query_pairs().count() == 1;
                is_video && is_single
            }
            _ => false,
        }
    }

    /// Returns the chapter album's title.
    pub fn title(&self) -> &str {
        &self.header.title
    }

    /// An iterator over the chapter tracks.
    pub fn tracks(&self) -> std::slice::Iter<'_, ChapterTrack> {
        self.tracks.iter()
    }
}

impl ChapterTrack {
    /// Return the track's title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return a tuple of `(start_time, end_time)`.
    pub fn range(&self) -> (f32, f32) {
        (self.start_time, self.end_time)
    }

    /// Construct a [`Tag`] representing the [`ChapterTrack`].
    ///
    /// `parent` is the chapter album spec, which the track is a part of, 
    /// `tracks` is a tuple of `(track_num, track_total)`.
    pub fn tag(&self, tracks: (u32, u32), parent: &Chapters) -> Tag {
        let year = parent.header.release_year.chars().take(4).collect();
        Tag {
            title: self.title.clone(),
            album_title: parent.header.title.clone(),
            track: tracks.0,
            track_total: tracks.1,
            genre: parent.header.genre.clone(),
            artists: self.artists.join(", "),
            album_artists: parent.header.album_artists.join(", "),
            release_year: year,
        }
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

/// The target format of audio files.
///
/// For now supported are FLAC and MP3 with their respective tag formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Format {
    Flac,
    Mp3,
}

impl Format {
    /// The `ffmpeg` audio codec for the format.
    pub fn codec(&self) -> &'static str {
        match self {
            Format::Flac => "flac",
            Format::Mp3 => "libmp3lame",
        }
    }

    /// The format's file extension
    pub fn ext(&self) -> &'static str {
        match self {
            Format::Flac => "flac",
            Format::Mp3 => "mp3",
        }
    }

    /// The format's preffered quality
    pub fn quality(&self) -> &'static str {
        match self {
            Format::Flac => "8",
            Format::Mp3 => "2",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Flac => write!(f, "FLAC"),
            Format::Mp3 => write!(f, "MP3"),
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
