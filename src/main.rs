// From Kleo's brain:
//
// In short, we need to do three "simple" things:
//
//  - [x] Set up a daemonic run loop
//  - [x] Poll MPRIS for now-playing information
//  - [x] Convert that info to Discord and send over RPC
//
// How hard could it possibly be?
// Update: not that bad, actually.

mod error;

use crate::error::OxiError;
use discord_presence::models::{ActivityType, DisplayType};
use discord_presence::{Client, Event};
use log::{debug, info, warn};
use mpris::{Metadata, MetadataValueKind, Player};
use std::thread::sleep;
use std::time;

#[derive(Debug, Clone)]
struct NowPlaying {
    title: String,
    artist: Option<String>,
    album: Option<String>,
}

enum MediaType {
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
}

impl TryFrom<Player> for NowPlaying {
    type Error = OxiError;

    fn try_from(value: Player) -> Result<Self, Self::Error> {
        let metadata: Metadata = value.get_metadata()?;
        debug!("{:?}", metadata);
        let metadata_h = MetadataHelper::new(metadata);
        Ok(Self {
            title: metadata_h.get_attribute("xesam:title")?,
            artist: metadata_h.get_attribute("xesam:artist").ok(),
            album: metadata_h.get_attribute("xesam:album").ok(),
        })
    }
}

fn main() {
    env_logger::init();
    let id: u64 = 1447306598555844844; // This isn't a secret, though it kinda feels like one

    info!("Starting DRPC client with discord...");
    let mut drpc = Client::new(id);
    let _ = drpc.on_ready(|_ctx| {
        info!("Ready, captain!");
    });

    drpc.start();
    drpc.block_until_event(Event::Ready)
        .expect("Failed to get Ready status");
    info!("Client started successfully.");

    let interval_ms = 1000;

    info!(
        "Polling MPRIS every {}ms ({}s)",
        interval_ms,
        interval_ms as f32 / 1000.0
    );

    let mut sleeping = false;

    // graceful shutdowns are for babies
    loop {
        sleep(time::Duration::from_millis(interval_ms));
        let finder = mpris::PlayerFinder::new().unwrap();
        let active_res = finder.find_active();
        if active_res.is_err() {
            if let Err(e) = drpc.clear_activity() {
                warn!("Failed to clear status: {e}")
            }
            continue;
        }
        let active = active_res.unwrap();

        let np_res: Result<NowPlaying, OxiError> = active.try_into();
        if let Err(e) = &np_res {
            // NowPlaying translation will sometimes fail when the user switches songs rapidly
            // I do that a lot, apparently, so I decree this should not be fatal.
            warn!("{e}");
            continue;
        }
        let np = np_res.unwrap();
        let message = match np.guess_type() {
            MediaType::Music => Some(format!("{} - {}", np.artist.unwrap_or_default(), np.title)),
            MediaType::Video => Some("Watching a video".to_string()),
            MediaType::Unknown => None,
        };

        match message {
            Some(msg) => {
                sleeping = false;
                drpc.set_activity(|act| {
                    act.state(msg)
                        .status_display(DisplayType::State)
                        .activity_type(ActivityType::Listening)
                })
                .expect("Failed to set activity");
            }
            None => {
                // We don't want to be continuously clearing status if it's already been cleared.
                // This should make oxi-san play nice while a game is running, but no music is playing.
                if sleeping {
                    debug!("I would clear status, but I am sleeping, so I instead will not do that.");
                    continue;
                }

                if let Err(e) = drpc.clear_activity() {
                    warn!("Failed to clear status: {e}")
                } else {
                    sleeping = true
                }
            }
        };
    }
}
