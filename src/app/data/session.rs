// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use egui::Vec2;
use egui::vec2;
use midi_msg::FileTimeSignature;
use midi_msg::Meta;
use midi_msg::MidiFile;

use crate::MidiFlipperError;
use crate::app::data::SessionTrack;

const MIDDLE_C: u8 = 60;

#[derive(Debug)]
pub struct Session {
    name: String,
    length: f32,
    midi_header: midi_msg::Header,
    tracks: Vec<SessionTrack>,
    marker_events: Vec<(f64, Meta)>,
    beats_paint_cache: Vec<([Vec2; 2], bool)>,

    center_note: u8,
    flip_bend: bool,
    is_placeholder: bool,
}

impl Session {
    pub fn new(name: String, midi_file: MidiFile) -> Result<Self, MidiFlipperError> {
        let validate = MidiFile::from_midi(&midi_file.to_midi());
        if validate.is_err() {
            return Err(MidiFlipperError::MidiValidationFailed);
        }

        let center_note = MIDDLE_C;
        let flip_bend = false;

        let mut tracks = Vec::with_capacity(midi_file.tracks.len());
        let mut length = 0.0;
        for track in midi_file.tracks {
            let session_track = SessionTrack::from_track(track, center_note, flip_bend);
            if session_track.length() > length {
                length = session_track.length();
            }
            tracks.push(session_track);
        }

        let mut this = Self {
            name,
            midi_header: midi_file.header,
            length,
            tracks,
            marker_events: vec![],
            beats_paint_cache: vec![],
            center_note,
            flip_bend,
            is_placeholder: false,
        };
        this.generate_bg_paint_cache();

        Ok(this)
    }

    pub fn from_file(file_path: PathBuf) -> Result<Self, MidiFlipperError> {
        let name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let bytes = std::fs::read(file_path)?;
        let midi_file = MidiFile::from_midi(&bytes)?;

        Self::new(name, midi_file)
    }

    pub fn placeholder() -> Self {
        Self {
            name: String::from("No File"),
            length: 0.0,
            midi_header: midi_msg::Header::default(),
            tracks: vec![],
            marker_events: vec![],
            beats_paint_cache: vec![],
            center_note: MIDDLE_C,
            flip_bend: false,
            is_placeholder: true,
        }
    }

    pub fn is_placeholder(&self) -> bool {
        self.is_placeholder
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn midi_header(&self) -> &midi_msg::Header {
        &self.midi_header
    }

    pub fn tracks(&self) -> &[SessionTrack] {
        &self.tracks
    }

    pub fn tracks_mut(&mut self) -> &mut [SessionTrack] {
        &mut self.tracks
    }

    pub fn beats_paint_cache(&self) -> &[([Vec2; 2], bool)] {
        &self.beats_paint_cache
    }

    pub fn marker_events(&self) -> &[(f64, Meta)] {
        &self.marker_events
    }

    pub fn center_note(&self) -> u8 {
        self.center_note
    }

    pub fn set_center_note(&mut self, center_note: u8) {
        self.center_note = center_note;
        for track in &mut self.tracks {
            track.set_center_note(center_note);
        }
    }

    pub fn reset_center_note(&mut self) {
        self.set_center_note(MIDDLE_C);
    }

    pub fn flip_bend(&self) -> bool {
        self.flip_bend
    }

    pub fn set_flip_bend(&mut self, flip_bend: bool) {
        self.flip_bend = flip_bend;
        for track in &mut self.tracks {
            track.set_flip_bend(flip_bend);
        }
    }

    pub fn assemble_flipped_midi(&self) -> MidiFile {
        let mut tracks = Vec::with_capacity(self.tracks.len());
        for track in &self.tracks {
            let midi_track = if track.flip_enabled() {
                track.track_flipped().midi_track().clone()
            } else {
                track.track_original().midi_track().clone()
            };
            tracks.push(midi_track);
        }

        MidiFile {
            header: self.midi_header.clone(),
            tracks,
        }
    }

    fn generate_bg_paint_cache(&mut self) {
        let midi_msg::Division::TicksPerQuarterNote(ticks_in_quarter) = self.midi_header.division
        else {
            println!("unhandled division");
            return;
        };
        let ticks_in_whole = ticks_in_quarter * 4;
        let mut cache = vec![];

        let mut time_signature = FileTimeSignature {
            numerator: 4,
            denominator: 4,
            clocks_per_metronome_tick: 24,
            thirty_second_notes_per_24_clocks: 8,
        };
        let mut note_len = ticks_in_whole / time_signature.denominator;
        let mut beat = 0;
        let mut next_note_time = 0.0;

        let mut tracks: Vec<_> = self
            .tracks
            .iter()
            .map(|t| t.track_original().midi_track().events().iter())
            .collect();

        let mut next_events: Vec<_> = tracks.iter_mut().map(|i| (0.0, i.next())).collect();

        loop {
            let Some(next_track) = ({
                let mut lowest_time = f64::INFINITY;
                let mut next_track = None;

                for (i, (prev_time, event)) in next_events.iter().enumerate() {
                    let Some(event) = event else {
                        continue;
                    };
                    let ev_time = prev_time + event.delta_time as f64;
                    if ev_time < lowest_time {
                        lowest_time = ev_time;
                        next_track = Some(i);
                    }
                }
                next_track
            }) else {
                break;
            };

            let (time, event) = &mut next_events[next_track];
            let track_event = event.as_deref().unwrap();

            *time += track_event.delta_time as f64;

            while *time > next_note_time {
                cache.push((
                    [
                        vec2(next_note_time as f32, 0.0),
                        vec2(next_note_time as f32, 1.0),
                    ],
                    beat == 0,
                ));
                next_note_time += note_len as f64;
                beat += 1;
                beat %= time_signature.denominator;
            }

            if let midi_msg::MidiMsg::Meta { msg } = &track_event.event {
                match msg {
                    Meta::TimeSignature(ts) => {
                        time_signature = ts.clone();
                        note_len = ticks_in_whole / time_signature.denominator;
                        beat = 0;

                        self.marker_events.push((*time, msg.clone()));
                    }
                    Meta::Marker(_) | Meta::EndOfTrack | Meta::SetTempo(_) => {
                        self.marker_events.push((*time, msg.clone()));
                    }
                    _ => (),
                }
            };

            *event = tracks[next_track].next();
        }

        self.beats_paint_cache = cache;
    }
}
