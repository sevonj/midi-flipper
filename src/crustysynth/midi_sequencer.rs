// SPDX-License-Identifier: AGPL-3.0-or-later

use midi_msg::MidiMsg;
use std::time::Duration;

use crate::crustysynth::midi_region::MidiRegion;
use crate::crustysynth::midi_sink::MidiSink;

pub type SeqTrackInfo = (usize, MidiRegion);

pub struct MidiSequencer {
    tracks: Vec<SeqTrackInfo>,
    song_duration: Duration,
    song_position: Duration,
}

impl MidiSequencer {
    pub fn new(midi: &[MidiRegion]) -> Self {
        let tracks = midi.iter().map(|region| (0, region.clone())).collect();

        let song_duration = midi
            .iter()
            .map(|region| region.duration())
            .max()
            .unwrap_or(Duration::ZERO);

        Self {
            tracks,
            song_duration,
            song_position: Duration::ZERO,
        }
    }

    pub fn end_of_sequence(&self) -> bool {
        self.song_position > self.song_duration
    }

    pub fn advance_time(&mut self, delta_t: Duration) {
        self.song_position += delta_t;
    }

    pub fn take_event(&mut self) -> Option<(usize, &MidiMsg)> {
        for (i, (track_pos, region)) in self.tracks.iter_mut().enumerate() {
            if *track_pos >= region.events().len() {
                continue;
            }
            let event = &region.events()[*track_pos];
            if self.song_position >= event.time {
                *track_pos += 1;
                return Some((i, &event.msg));
            }
        }
        None
    }

    pub const fn song_duration(&self) -> Duration {
        self.song_duration
    }

    pub const fn song_position(&self) -> Duration {
        self.song_position
    }

    pub fn seek_to<R>(&mut self, event_sinks: &mut [R], pos: Duration)
    where
        R: MidiSink,
    {
        if pos < self.song_position {
            self.song_position = Duration::ZERO;
            for (pos, _) in &mut self.tracks {
                *pos = 0;
            }
            for sink in event_sinks {
                sink.reset();
            }
        }

        let delta_t = pos.saturating_sub(self.song_position);
        if delta_t > Duration::ZERO {
            self.advance_time(delta_t);
        }
    }
}
