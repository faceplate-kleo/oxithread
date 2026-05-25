use crate::error::OxiError;
use crate::now_playing::NowPlaying;
use log::{debug, error};
use std::time;
use std::time::Duration;
use time::Instant;

#[cfg(target_os = "linux")]
type SessionPlayer = mpris::Player;
#[cfg(target_os = "windows")]
type SessionPlayer = windows::Media::Control::GlobalSystemMediaTransportControlsSession;

#[derive(Debug)]
pub struct StatusTracker {
    pub now_playing: Option<NowPlaying>,

    pub player: Option<SessionPlayer>,

    pub transition_time: Instant,
    pub expected_end: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct _StatusBar {
    pub start: Instant,
    pub position: Duration,
    pub end: Option<Instant>,
}

impl StatusTracker {
    pub fn new(now_playing: Option<NowPlaying>, player: Option<SessionPlayer>) -> Self {
        let transition_time = Instant::now();
        let expected_end = match &now_playing {
            Some(now_playing) => match now_playing.length {
                Some(len) => {
                    let dur = Duration::from_micros(len as u64);
                    Some(transition_time + dur)
                }
                None => None,
            },
            None => None,
        };

        Self {
            now_playing,
            player,
            transition_time,
            expected_end,
        }
    }

}

// impl _StatusBar {
//     pub fn _time_status_as_string(&self) -> Option<String> {
//         self.end?;
//         let total_duration = self.end.unwrap() - self.start;
//         error!("{}", total_duration.as_secs());
//
//         let current = Self::_duration_to_string(self.position);
//         let total = Self::_duration_to_string(total_duration);
//
//         let stamp = format!("{} / {}", current, total).to_string();
//
//         debug!("{}", stamp);
//
//         Some(stamp)
//     }
//
//     fn _duration_to_string(dur: Duration) -> String {
//         let total_seconds = dur.as_secs();
//
//         let minutes = total_seconds / 60;
//         let seconds = total_seconds % 60;
//
//         format!("{}:{:02}", minutes, seconds).to_string()
//     }
// }
//
// impl TryFrom<&StatusTracker> for _StatusBar {
//     type Error = OxiError;
//
//     fn try_from(value: &StatusTracker) -> Result<Self, Self::Error> {
//         if let Some(player) = &value.player {
//             return Ok(Self {
//                 start: value.transition_time,
//                 position: player.get_position()?,
//                 end: value.expected_end,
//             });
//         }
//         debug!("Failed to coerce StatusBar from StatusTracker");
//         Err(OxiError::_StatusBarCoercion)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn some_durations_get_formatted_right() {
//         // perhaps the most basic sanity check
//         let a = Duration::from_secs(0);
//         assert_eq!(_StatusBar::_duration_to_string(a), "0:00");
//         let b = Duration::from_secs(90);
//         assert_eq!(_StatusBar::_duration_to_string(b), "1:30");
//         let c = Duration::from_secs(120);
//         assert_eq!(_StatusBar::_duration_to_string(c), "2:00");
//         let d = Duration::from_secs(121);
//         assert_eq!(_StatusBar::_duration_to_string(d), "2:01");
//     }
// }
