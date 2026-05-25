#[cfg(target_os = "linux")]
use mpris::{Metadata, MetadataValueKind, Player};

#[cfg(target_os = "linux")]
struct MetadataHelper {
    metadata: Metadata,
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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
