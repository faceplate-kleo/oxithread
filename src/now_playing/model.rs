use crate::error::OxiError;
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlaying {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub length: Option<i64>,
}

pub enum MediaType {
    Music,
    Video,
    Unknown,
}

impl NowPlaying {
    pub fn guess_type(&self) -> MediaType {
        let artist = self.artist.clone().unwrap_or_default();
        let album = self.album.clone().unwrap_or_default();
        if !artist.is_empty() && !album.is_empty() {
            MediaType::Music
        } else if !artist.is_empty() && album.is_empty() {
            MediaType::Video
        } else {
            MediaType::Unknown
        }
    }
}
