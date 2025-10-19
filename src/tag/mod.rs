mod chapters;
mod playlist;

use std::fs;
use std::path::{Path, PathBuf};

use id3::frame::{Picture, PictureType as ID3PictureType};
use id3::{Content, Frame, TagLike, Version};
use metaflac::block::PictureType as FLACPictureType;
use url::Url;

use crate::specfile::Format;

/// A structure representing tag data.
///
/// This struct is the same regardless of the target tag - we are only interested in things which
/// both tag formats support.
pub struct Tag {
    /// The track title.
    pub title: String,
    /// The title of the album.
    pub album_title: String,
    /// Track index.
    pub track: u32,
    /// Total tracks in the album.
    pub track_total: u32,
    /// Genre string. DO NOT use ID3 genre indices.
    pub genre: String,
    /// Contributing artists concatenated with commas.
    pub artists: String,
    /// Album artists concatenated with commas.
    pub album_artists: String,
    /// Album release year, for ID3 tags it may be skipped because we have a beef with ID3.
    pub release_year: String,
}

impl Tag {
    pub fn write_tag(
        self,
        path: impl AsRef<Path>,
        target: Format,
        cover: Vec<u8>,
        mime: &'static str,
    ) -> crate::Result<()> {
        match target {
            Format::Flac => self.tag_flac(path, cover, mime),
            Format::Mp3 => self.tag_mp3(path, cover, mime),
        }
    }

    fn tag_flac(
        self,
        path: impl AsRef<Path>,
        cover: Vec<u8>,
        mime: &'static str,
    ) -> crate::Result<()> {
        let mut tag = metaflac::Tag::read_from_path(path).map_err(|e| TagError::from(e))?;
        let meta = tag.vorbis_comments_mut();
        meta.set_title(vec![self.title]);
        meta.set_album(vec![self.album_title]);
        meta.set_track(self.track);
        meta.set_total_tracks(self.track_total);
        meta.set_genre(vec![self.genre]);
        meta.set_artist(vec![self.artists]);
        meta.set_album_artist(vec![self.album_artists]);
        meta.set("DATE", vec![self.release_year]);

        tag.add_picture(mime, FLACPictureType::CoverFront, cover);

        tag.save().map_err(|e| TagError::from(e))?;

        Ok(())
    }

    fn tag_mp3(
        self,
        path: impl AsRef<Path>,
        cover: Vec<u8>,
        mime: &'static str,
    ) -> crate::Result<()> {
        let mut tag = id3::Tag::new();
        tag.set_title(self.title);
        tag.set_album(self.album_title);
        tag.set_track(self.track);
        tag.set_total_tracks(self.track_total);
        tag.set_genre(self.genre);
        tag.set_artist(self.artists);
        tag.set_album_artist(self.album_artists);
        // we most likely have a valid i32, but I don't want to risk an unwrap
        if let Ok(y) = self.release_year.parse::<i32>() {
            let year = id3::Timestamp { year: y, ..Default::default() };
            // Spam all dates, maybe one will work, I hate this
            tag.set_date_recorded(year);
            tag.set_date_released(year);
            tag.set_original_date_released(year);
        }

        let pic = Picture {
            mime_type: mime.to_string(),
            picture_type: ID3PictureType::CoverFront,
            description: "FRONT_COVER".to_string(),
            data: cover,
        };

        tag.add_frame(Frame::with_content("APIC", Content::Picture(pic)));

        tag.write_to_path(path, Version::Id3v23)
            .map_err(|e| TagError::from(e))?;

        Ok(())
    }
}

/// Fetch the cover from the web or the filesystem, return a tuple of (`bytes`, `mime_type`).
fn get_cover(cover: &Url) -> crate::Result<(Vec<u8>, &'static str)> {
    if cover.scheme() == "file" {
        let path = cover
            .to_file_path()
            .map_err(|_| TagError::BadPath(cover.clone()))?;
        log::debug!("Reading cover data from {}", path.display());
        let data = fs::read(&path)?;
        let mime = infer::get(&data)
            .ok_or_else(|| TagError::UnknownMime(format!("{}", path.display())))?;
        Ok((data, mime.mime_type()))
    } else {
        log::debug!("Fetching cover art from {}", cover);
        let response = reqwest::blocking::get(cover.clone()).map_err(|e| TagError::from(e))?;
        let data = response.bytes().map_err(|e| TagError::from(e))?;
        let mime = infer::get(&data).ok_or_else(|| TagError::UnknownMime(cover.to_string()))?;
        Ok((data.to_vec(), mime.mime_type()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("unknown mime type of `{0}`")]
    UnknownMime(String),
    #[error("cannot determine the file path of `{0}`")]
    BadPath(Url),
    #[error("IO error when {0}")]
    IoError(&'static str, #[source] std::io::Error),
    #[error("cannot tag `{0}`: file not found")]
    FileNotFound(PathBuf, #[source] std::io::Error),
    #[error("cannot tag `{0}`: broken symlink")]
    BrokenPath(PathBuf),
    #[error("cannot tag {0}: cannot find the input files directory `{1}`")]
    NoIndir(String, PathBuf),
    #[error("fetching the cover failed")]
    FetchError(#[from] reqwest::Error),
    #[error("writing FLAC tag failed")]
    FlacError(#[from] metaflac::Error),
    #[error("writing ID3 tag failed")]
    ID3Error(#[from] id3::Error),
}
