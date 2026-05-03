// SPDX-License-Identifier: AGPL-3.0-or-later

use midi_msg::Division;
use midi_msg::Header;
use midi_msg::Meta;
use midi_msg::MidiFile;
use midi_msg::MidiMsg;
use midi_msg::TimeCodeType;
use std::time::Duration;

use crate::crustysynth::midi_region::MidiEvent;
use crate::crustysynth::midi_region::MidiRegion;
use crate::crustysynth::midi_sink::MidiSink;

pub type SeqTrackInfo = (usize, MidiRegion);

pub struct MidiSequencer {
    tracks: Vec<SeqTrackInfo>,
    song_duration: Duration,
    song_position: Duration,
}

impl MidiSequencer {
    pub fn new(midi_file: &MidiFile) -> Self {
        let tracks: Vec<SeqTrackInfo> = generate_midi_regions(midi_file)
            .into_iter()
            .map(|region| (0, region))
            .collect();

        let song_duration = tracks
            .iter()
            .map(|(_, region)| region.duration())
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

fn tick_duration(header: &Header, bpm: f64) -> Duration {
    let in_secs = match header.division {
        Division::TicksPerQuarterNote(ticks) => 60.0 / bpm / f64::from(ticks),
        Division::TimeCode {
            frames_per_second,
            ticks_per_frame,
        } => {
            let fps = match frames_per_second {
                TimeCodeType::FPS24 => 24.0,
                TimeCodeType::FPS25 => 25.0,
                TimeCodeType::DF30 | TimeCodeType::NDF30 => 30.0,
            };
            1.0 / fps / f64::from(ticks_per_frame)
        }
    };
    Duration::from_secs_f64(in_secs)
}

fn generate_midi_regions(midi_file: &MidiFile) -> Vec<MidiRegion> {
    let midi_header = &midi_file.header;
    let midi_tracks = &midi_file.tracks;
    let num_tracks = midi_tracks.len();

    let mut track_positions: Vec<usize> = vec![0; num_tracks];
    let mut track_ticks: Vec<u32> = vec![0; num_tracks];
    let mut track_times: Vec<Duration> = vec![Duration::ZERO; num_tracks];

    let mut bpm: f64 = 120.0;

    let mut abs_tracks: Vec<Vec<MidiEvent>> = vec![vec![]; num_tracks];
    while let Some(track_index) = {
        let mut lowest_tick = u32::MAX;
        let mut next_track = None;

        for (i, pos) in track_positions.iter().enumerate() {
            let track = midi_tracks[i].events();
            if *pos >= track.len() {
                continue;
            }

            let event = &track[*pos];
            let event_tick = track_ticks[i] + event.delta_time;
            if event_tick < lowest_tick {
                lowest_tick = event_tick;
                next_track = Some(i);
            }
        }

        next_track
    } {
        let event_index = track_positions[track_index];
        let track = midi_tracks[track_index].events();
        let track_event = &track[event_index];

        track_positions[track_index] += 1;
        track_ticks[track_index] += track_event.delta_time;
        track_times[track_index] += tick_duration(midi_header, bpm) * track_event.delta_time;

        abs_tracks[track_index].push(MidiEvent::new(
            track_times[track_index],
            track_event.event.clone(),
        ));

        if let MidiMsg::Meta {
            msg: Meta::SetTempo(tempo),
        } = track_event.event
        {
            bpm = 60_000_000.0 / f64::from(tempo);
        }
    }

    let mut regions = Vec::with_capacity(abs_tracks.len());
    for events in abs_tracks {
        regions.push(MidiRegion::new(events));
    }
    regions
}
