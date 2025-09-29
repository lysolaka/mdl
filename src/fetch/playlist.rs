use serde::ser::{Error, SerializeStruct};
use url::Url;

/// A structure representing a fetched playlist and its interesting data.
///
/// It should only be used to fetch and to save to a specfile. Deserializing a specfile into this
/// struct is an error because the [`serde::Serialize`] and [`serde::Deserialize`] implementations
/// are not reciprocal.
#[derive(Debug, serde::Deserialize)]
pub struct Playlist {
    id: String,
    title: String,
    #[serde(skip_deserializing)]
    release_year: String,
    #[serde(default)]
    album_artists: Vec<String>,
    #[serde(skip_deserializing)]
    genre: String,
    #[serde(rename(deserialize = "playlist_count"))]
    track_total: u32,
    #[serde(skip_deserializing)]
    cover: String,
    #[serde(rename(deserialize = "entries"))]
    tracks: Vec<Track>,
}

#[derive(Debug, serde::Deserialize)]
struct Track {
    id: String,
    extractor: String,
    title: String,
    #[serde(default)]
    artists: Vec<String>,
    #[serde(rename = "playlist_index")]
    track_num: u32,
}

impl serde::Serialize for Playlist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct Header<'a> {
            id: &'a str,
            title: &'a str,
            release_year: &'a str,
            album_artists: &'a [String],
            genre: &'a str,
            track_total: &'a u32,
            cover: &'a str,
        }

        let header = Header {
            id: &self.id,
            title: &self.title,
            release_year: &self.release_year,
            album_artists: self.album_artists.as_slice(),
            genre: &self.genre,
            track_total: &self.track_total,
            cover: &self.cover,
        };

        let mut state = serializer.serialize_struct("Playlist", 2)?;
        state.serialize_field("playlist", &header)?;
        state.serialize_field("track", &self.tracks)?;
        state.end()
    }
}

impl serde::Serialize for Track {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let url = match self.extractor.as_str() {
            "youtube" => format!("https://www.youtube.com/watch?v={}", self.id),
            "soundcloud" => format!("https://api-v2.soundcloud.com/tracks/{}", self.id),
            other => {
                return Err(S::Error::custom(format!(
                    "unsupported extractor: {}",
                    other
                )));
            }
        };
        let mut state = serializer.serialize_struct("Track", 4)?;
        state.serialize_field("url", &url)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("artists", &self.artists)?;
        state.serialize_field("track_num", &self.track_num)?;
        state.end()
    }
}

/// Fetch a playlist spec from `url`.
///
/// Supported URLs are:
/// - YouTube playlists
/// - YouTube videos inside a playlist (the entire playlist will be fetched)
/// - Soundcloud playlists (sets)
///
/// The `id` is a user identifier - it has nothing to do with the actual content,
/// the user may set it to `Some(id)` and that `id` will be used by the program to
/// name the spec. Setting `id` to `None` will automatically assign an `id`.
pub fn playlist(url: &Url, id: Option<&str>) -> crate::Result<Playlist> {
    log::debug!("Getting playlist URL of {}", url);
    let url = super::get_playlist_url(url, true)?;
    log::info!("Fetching playlist info of {}", url);
    let info = super::fetch(&url)?;
    log::info!("Finished fetching {}", url);
    match id {
        Some(id) => {
            log::trace!("Setting ID to {} for {}", id, url);
            Ok(Playlist {
                id: id.to_string(),
                ..info
            })
        }
        None => {
            log::trace!("No ID provided for {}, using {}", url, info.id);
            Ok(info)
        },
    }
}
