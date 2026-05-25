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
mod now_playing;
mod status;

use crate::error::OxiError;

use crate::now_playing::{MediaType, NowPlaying};

use crate::status::StatusTracker;
use discord_presence::models::{ActivityType, DisplayType};
use discord_presence::{Client, Event};
use log::{debug, info, warn};
use std::thread::sleep;
use std::time;
use windows::Media::Control::{GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager};

#[tokio::main]
async fn main() {
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

    #[cfg(target_os = "windows")]
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().unwrap().await.unwrap();

    let mut tracker = StatusTracker::new(None, None);
    // graceful shutdowns are for babies
    loop {
        sleep(time::Duration::from_millis(interval_ms));

        #[cfg(target_os = "linux")]
        let np_res: Result<NowPlaying, OxiError> = {
            let finder = mpris::PlayerFinder::new().unwrap();
            let active_res = finder.find_active();
            if active_res.is_err() {
                if let Err(e) = drpc.clear_activity() {
                    warn!("Failed to clear status: {e}")
                }
                continue;
            }
            let active = active_res.unwrap()
            (&active).try_into();
        };

        #[cfg(target_os = "windows")]
        let np_res: Result<(NowPlaying, GlobalSystemMediaTransportControlsSession), OxiError> = {
            let session = manager.GetCurrentSession().unwrap();
            Ok((NowPlaying::from_session(&session).await.unwrap(), session))
        };

        if let Err(e) = &np_res {
            // NowPlaying translation will sometimes fail when the user switches songs rapidly
            // I do that a lot, apparently, so I decree this should not be fatal.
            warn!("{e}");
            continue;
        }
        let np_tup = np_res.unwrap();

        let np = np_tup.0;
        let active = np_tup.1;


        if let Some(tracker_np) = &tracker.now_playing
            && tracker_np == &np
        {
            debug!("Media hasn't changed, continuing...");
            continue;
        }

        tracker = StatusTracker::new(Some(np.clone()), Some(active));

        // Timestamp stuff just seems way too unstable with Chromium
        // It's probably fine with other media players, but I use Chromium. Sorry not sorry.
        //
        // let timestamp = StatusBar::try_from(&tracker).unwrap().time_status_as_string().unwrap_or("unknown timestamp".to_string());

        let message = match np.guess_type() {
            MediaType::Music => Some(format!("{} - {}", np.artist.unwrap_or_default(), np.title)),
            MediaType::Video => Some("Watching a video".to_string()),
            MediaType::Unknown => None,
        };

        match message {
            Some(msg) => {
                drpc.set_activity(|act| {
                    act.details(msg)
                        .state(np.album.unwrap_or("Unknown Album".to_string()))
                        .append_buttons(|button| {
                            button
                                .label("DONT CLICK")
                                .url("https://www.youtube.com/watch?v=E4WlUXrJgy4")
                        })
                        .activity_type(ActivityType::Listening)
                        .status_display(DisplayType::Details)
                })
                .expect("Failed to set activity");
            }
            None => {
                if let Err(e) = drpc.clear_activity() {
                    warn!("Failed to clear status: {e}")
                }
                tracker.now_playing = None;
            }
        };
    }
}
