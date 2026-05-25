use windows::Media::Control::{GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionMediaProperties};
use crate::error::OxiError;
use crate::now_playing::NowPlaying;

#[cfg(target_os = "windows")]
impl NowPlaying {
    pub async fn from_session(session: &GlobalSystemMediaTransportControlsSession) -> Result<Self, OxiError> {
        let prop = session.TryGetMediaPropertiesAsync().unwrap().await.unwrap();
        prop.try_into()
    }
}

#[cfg(target_os = "windows")]
impl TryFrom<GlobalSystemMediaTransportControlsSessionMediaProperties> for NowPlaying {
    type Error = OxiError;

    fn try_from(value: GlobalSystemMediaTransportControlsSessionMediaProperties) -> Result<Self, Self::Error> {

        let artist_dash_album = value.Artist().unwrap().to_string();

        let artist_split_album: Vec<&str> = artist_dash_album.split(" — ").collect(); // TODO this is just for Apple Music
        let artist = artist_split_album.first().unwrap_or(&"").to_string();
        let album = artist_split_album.last().unwrap_or(&"").to_string();

        Ok(NowPlaying{
            title: value.Title().unwrap().to_string(),
            artist: Some(artist),
            album: Some(album),
            length: None,
        })
    }
}
