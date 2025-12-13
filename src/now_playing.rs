use crate::error::OxiError;
use log::debug;
use mpris::{Metadata, MetadataValueKind, Player};

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

struct MetadataHelper {
    metadata: Metadata,
}

impl MetadataHelper {
    pub fn new(metadata: Metadata) -> Self {
        Self { metadata }
    }
    pub fn get_attribute(&self, key: &str) -> Result<String, OxiError> {
        if let Some(val) = self.metadata.get(key) {
            return match val.kind() {
                MetadataValueKind::String => Ok(val.clone().into_string().unwrap()),
                MetadataValueKind::Array => {
                    let as_arr = val.as_array().unwrap();
                    Ok(as_arr
                        .iter()
                        .map(|v| v.as_string().unwrap().clone())
                        .collect::<Vec<String>>()
                        .join(","))
                }
                _ => Err(OxiError::Translation),
            };
        }
        Err(OxiError::MissingMetadataField(key.to_string()))
    }

    pub fn get_integer_attribute(&self, key: &str) -> Result<i64, OxiError> {
        if let Some(val) = self.metadata.get(key) {
            return match val.kind() {
                MetadataValueKind::I64 => {
                    let as_i64 = val.clone().into_i64().unwrap();
                    if as_i64 == i64::MAX {
                        return Err(OxiError::Translation);
                    }
                    Ok(as_i64)
                }
                _ => Err(OxiError::Translation),
            };
        }
        Err(OxiError::MissingMetadataField(key.to_string()))
    }
}

impl TryFrom<&Player> for NowPlaying {
    type Error = OxiError;

    fn try_from(value: &Player) -> Result<Self, Self::Error> {
        let metadata: Metadata = value.get_metadata()?;
        debug!("{:?}", metadata);
        let metadata_h = MetadataHelper::new(metadata);
        Ok(Self {
            title: metadata_h.get_attribute("xesam:title")?,
            artist: metadata_h.get_attribute("xesam:artist").ok(),
            album: metadata_h.get_attribute("xesam:album").ok(),
            length: metadata_h.get_integer_attribute("mpris:length").ok(),
        })
    }
}
