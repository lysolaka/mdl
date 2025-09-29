use serde::ser::{Error, SerializeStruct};
use url::Url;

/// A struct representing a fetched album from signle video's chapters.
///
/// It should only be used to fetch and to save to a specfile. Deserializing a specfile into this
/// struct is an error because the [`serde::Serialize`] and [`serde::Deserialize`] implementations
/// are not reciprocal.
#[derive(Debug, serde::Deserialize)]
pub struct Chapters {
    id: String,
    #[serde(rename(deserialize = "original_url"))]
    url: String,
    title: String,
    #[serde(skip_deserializing)]
    release_year: String,
    #[serde(skip_deserializing)]
    album_artists: Vec<String>,
    #[serde(skip_deserializing)]
    genre: String,
    #[serde(skip_deserializing)]
    cover: String,
    #[serde(rename(deserialize = "chapters"))]
    tracks: Option<Vec<Track>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Track {
    title: String,
    #[serde(skip_deserializing)]
    artists: Vec<String>,
    start_time: f32,
    end_time: f32,
}

impl serde::Serialize for Chapters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct Header<'a> {
            id: &'a str,
            url: &'a str,
            title: &'a str,
            release_year: &'a str,
            album_artists: &'a [String],
            genre: &'a str,
            cover: &'a str,
        }

        let header = Header {
            id: &self.id,
            url: &self.url,
            title: &self.title,
            release_year: &self.release_year,
            album_artists: self.album_artists.as_slice(),
            genre: &self.genre,
            cover: &self.cover,
        };

        let mut state = serializer.serialize_struct("Chapters", 2)?;
        state.serialize_field("chapters", &header)?;
        state.serialize_field("track", &self.tracks)?;
        state.end()
    }
}

/// Fetch a chapters spec from `url`.
///
/// Supported URLs are:
/// - YouTube videos
/// - YouTube videos inside a playlist (only the video will be fetched)
///
/// The `id` is a user identifier - it has nothing to do with the actual content,
/// the user may set it to `Some(id)` and that `id` will be used by the program to
/// name the spec. Setting `id` to `None` will automatically assign an `id`.
pub fn chapters(url: &Url, id: Option<&str>) -> crate::Result<Chapters> {
    log::debug!("Getting video URL of {}", url);
    let url = match super::get_video_url(url) {
        Ok(url) => url,
        Err(e) => {
            log::error!("{}", e);
            return Err(e.into());
        },
    };
    log::info!("Fetching video info of {}", url);
    let info: Chapters = super::fetch(&url)?;
    log::info!("Finished fetching {}", url);
    if info.tracks.is_none() {
        log::warn!("The video {} does not contain any chapters", url);
    }
    match id {
        Some(id) => {
            log::trace!("Setting ID to {} for {}", id, url);
            Ok(Chapters {
                id: id.to_string(),
                ..info
            })
        }
        None => {
            log::trace!("No ID provided for {}, using {}", url, info.id);
            Ok(info)
        }
    }
}

/// Fetch multiple chapters specs from `url`.
///
/// Supported URLs are:
/// - YouTube playlists
/// - YouTube videos inside a playlist (every video in the playlist will be fetched)
///
/// The `id` is a user identifier - it has nothing to do with the actual content,
/// the user may set it to `Some(id)` and that `id` will be used by the program to
/// name the spec. Setting `id` to `None` will automatically assign an `id`.
///
/// Because the ID is a single string (for example "foo"), each spec's ID will be
/// "foo_<index>", where `index` is the video's playlist index.
pub fn chapters_recursive(url: &Url, id: Option<&str>) -> crate::Result<Vec<Chapters>> {
    // helper struct
    #[derive(serde::Deserialize)]
    struct ChaptersRecursive {
        entries: Vec<Chapters>,
    }

    log::debug!("Getting playlist URL of {}", url);
    let url = match super::get_playlist_url(url, false) {
        Ok(url) => url,
        Err(e) => {
            log::error!("{}", e);
            return Err(e.into());
        },
    };
    log::info!("Fetching playlist info of {}", url);
    let ChaptersRecursive { entries } = super::fetch(&url)?;
    log::info!("Finished fetching {}", url);
    match id {
        Some(id) => {
            let entries = entries
                .into_iter()
                .filter(|s| {
                    if s.tracks.is_none() {
                        log::warn!("Skipping {}: the video does not contain any chapters", url);
                        false
                    } else {
                        true
                    }
                })
                .enumerate()
                .map(|(i, s)| {
                    let id = format!("{}_{}", id, i);
                    log::trace!("Setting ID to {} for {}", id, &s.url);
                    Chapters { id, ..s }
                })
                .collect();
            Ok(entries)
        }
        None => {
            log::trace!("No ID provided for {}, using defaults", url);
            Ok(entries)
        }
    }
}
