// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Duration;

use midi_msg::Division;
use midi_msg::Header;
use midi_msg::MidiFile;
use midi_msg::MidiMsg;
use midi_msg::TimeCodeType;
use midi_msg::Track;
use midi_msg::TrackEvent;

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub time: Duration,
    pub tick: u32,
    pub msg: MidiMsg,
}

impl MidiEvent {
    pub fn new(time: Duration, tick: u32, msg: MidiMsg) -> Self {
        Self { time, tick, msg }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MidiRegion {
    events: Vec<MidiEvent>,
}

impl MidiRegion {
    pub fn new(events: Vec<MidiEvent>) -> Self {
        Self { events }
    }

    pub fn events(&self) -> &[MidiEvent] {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut [MidiEvent] {
        &mut self.events
    }

    pub fn duration(&self) -> Duration {
        self.events
            .last()
            .map(|last| last.time)
            .unwrap_or(Duration::ZERO)
    }

    pub fn from_midi_file(midi_file: &MidiFile) -> Vec<Self> {
        let midi_header = &midi_file.header;
        let midi_tracks = &midi_file.tracks;
        let num_tracks = midi_tracks.len();

        let mut track_positions: Vec<usize> = vec![0; num_tracks];
        let mut track_ticks: Vec<u32> = vec![0; num_tracks];
        let mut last_event_tick = 0;
        let mut time = Duration::ZERO;

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
            time += tick_duration(midi_header, bpm) * (track_ticks[track_index] - last_event_tick);
            last_event_tick = track_ticks[track_index];

            abs_tracks[track_index].push(MidiEvent::new(
                time,
                track_ticks[track_index],
                track_event.event.clone(),
            ));

            if let MidiMsg::Meta {
                msg: midi_msg::Meta::SetTempo(tempo),
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

    pub fn to_midi_track(&self) -> Track {
        let mut events = Vec::with_capacity(self.events().len());
        let mut tick = 0;
        for event in self.events() {
            let delta_time = event.tick - tick;
            tick = event.tick;
            events.push(TrackEvent {
                delta_time,
                event: event.msg.clone(),
                beat_or_frame: 0.0,
            });
        }

        Track::Midi(events)
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
