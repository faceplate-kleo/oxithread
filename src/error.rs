use thiserror::Error;

#[derive(Error, Debug)]
pub enum OxiError {
    #[error("Failed to translate MPRIS player data to NowPlaying")]
    Translation,

    #[error("Missing metadata field {0}")]
    MissingMetadataField(String),

    #[error("Failed DBus connection: {0}")]
    DBus(#[from] mpris::DBusError),

    #[error("Attempted to create status bar while nothing is playing")]
    _StatusBarCoercion,
}
