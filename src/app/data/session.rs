// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use egui::Vec2;
use egui::vec2;
use midi_msg::Division;
use midi_msg::FileTimeSignature;
use midi_msg::Meta;
use midi_msg::MidiFile;
use midi_msg::TimeCodeType;
use rustysynth::SoundFont;

use crate::MidiFlipperError;
use crate::app::data::SessionTrack;
use crate::crustysynth::CrustySynth;

#[derive(Debug, Clone)]
pub struct CachedBar {
    #[allow(dead_code)]
    pub start_time: f32,
    pub start_position: f32,
    pub end_time: f32,
    pub end_position: f32,
}

pub struct Session {
    name: String,
    length: f32,
    midi_header: midi_msg::Header,
    tracks: Vec<SessionTrack>,
    marker_events: Vec<(f64, Meta)>,
    beats_paint_cache: Vec<([Vec2; 2], bool)>,
    cached_bars: Vec<CachedBar>,

    global_transpose: i32,
    flip_bend: bool,
    is_placeholder: bool,

    synth: CrustySynth,
    playback_original: bool,
    selected_bar: usize,
    custom_soundfont: Option<Arc<SoundFont>>,
}

impl Session {
    pub fn new(name: String, midi_file: MidiFile) -> Result<Self, MidiFlipperError> {
        let validate = MidiFile::from_midi(&midi_file.to_midi());
        if validate.is_err() {
            return Err(MidiFlipperError::MidiValidationFailed);
        }

        let global_transpose = 0;
        let flip_bend = false;

        let mut tracks = Vec::with_capacity(midi_file.tracks.len());
        let mut length = 0.0;
        for track in midi_file.tracks {
            let session_track = SessionTrack::from_track(track, global_transpose, flip_bend);
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
            cached_bars: vec![],
            global_transpose,
            flip_bend,
            is_placeholder: false,
            synth: Default::default(),
            playback_original: false,
            selected_bar: 0,
            custom_soundfont: None,
        };
        this.generate_cache();
        this.refresh_synth();

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
            cached_bars: vec![],
            global_transpose: 0,
            flip_bend: false,
            is_placeholder: true,
            synth: Default::default(),
            playback_original: false,
            selected_bar: 0,
            custom_soundfont: None,
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

    pub fn cached_bars(&self) -> &[CachedBar] {
        &self.cached_bars
    }

    pub fn marker_events(&self) -> &[(f64, Meta)] {
        &self.marker_events
    }

    pub fn global_transpose(&self) -> i32 {
        self.global_transpose
    }

    pub fn set_global_transpose(&mut self, global_transpose: i32) {
        self.global_transpose = global_transpose;
        for track in &mut self.tracks {
            track.set_global_transpose(global_transpose);
        }
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

    pub fn check_for_changes(&mut self) {
        let mut has_changes = false;
        for track in &mut self.tracks {
            has_changes |= track.clear_changed();
        }
        if has_changes && !self.playback_original {
            self.refresh_synth();
        }
    }

    fn refresh_synth(&mut self) {
        let midi_file = if self.playback_original {
            self.assemble_original_midi()
        } else {
            self.assemble_flipped_midi()
        };
        if self.is_playback_in_progress() {
            self.synth.swap_midi_file(Arc::new(midi_file));
        } else {
            self.synth.set_midi_file(Some(Arc::new(midi_file)));
        }
    }

    pub fn playback_original(&self) -> bool {
        self.playback_original
    }

    pub fn set_playback_original(&mut self, playback_original: bool) {
        self.playback_original = playback_original;
        self.refresh_synth();
    }

    pub fn cursor_time(&self) -> f32 {
        if self.selected_bar >= self.cached_bars.len() {
            return self
                .cached_bars
                .last()
                .map(|bar| bar.start_time)
                .unwrap_or_default();
        }
        self.cached_bars[self.selected_bar].start_time
    }

    pub fn cursor_pos(&self) -> f32 {
        if self.selected_bar >= self.cached_bars.len() {
            return self
                .cached_bars
                .last()
                .map(|bar| bar.start_position)
                .unwrap_or_default();
        }
        self.cached_bars[self.selected_bar].start_position
    }

    pub fn set_cursor_pos(&mut self, cursor_pos: f32) {
        fn find(cursor_pos: f32, cached_bars: &[CachedBar]) -> usize {
            for (i, bar) in cached_bars.iter().enumerate() {
                if cursor_pos < bar.end_position {
                    return i;
                }
            }
            0
        }
        let index = find(cursor_pos, &self.cached_bars);
        self.selected_bar = index;

        if self.is_playback_in_progress() {
            self.synth.seek_to(self.cursor_time());
        }
    }

    pub fn is_playing(&self) -> bool {
        self.synth.is_playing()
    }

    pub fn is_playback_in_progress(&self) -> bool {
        self.synth.is_playback_in_progress()
    }

    pub fn playback_duration(&self) -> Duration {
        self.synth.duration()
    }

    pub fn playback_position(&self) -> Duration {
        self.synth.position()
    }

    pub fn set_playback_volume(&mut self, volume: f32) {
        self.synth.set_volume(volume)
    }

    pub fn set_custom_soundfont(&mut self, custom_soundfont: Option<Arc<SoundFont>>) {
        if let Some(soundfont) = custom_soundfont.as_ref() {
            self.synth.set_soundfont(soundfont.clone());
        } else {
            self.synth.set_soundfont(CrustySynth::default_soundfont());
        }
        self.custom_soundfont = custom_soundfont;
    }

    pub fn play(&mut self) {
        if self.synth.midi_file().is_none() {
            self.refresh_synth();
        }
        let in_progress = self.is_playback_in_progress();
        self.synth.play();
        if !in_progress {
            self.synth.seek_to(self.cursor_time());
        }
    }

    pub fn pause(&mut self) {
        self.synth.pause();
    }

    pub fn stop(&mut self) {
        self.synth.stop();
    }

    pub fn assemble_original_midi(&self) -> MidiFile {
        let mut tracks = Vec::with_capacity(self.tracks.len());
        for track in &self.tracks {
            let midi_track = track.track_original().midi_track().clone();
            tracks.push(midi_track);
        }

        MidiFile {
            header: self.midi_header.clone(),
            tracks,
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

    fn generate_cache(&mut self) {
        let ticks_in_whole = {
            let midi_msg::Division::TicksPerQuarterNote(ticks_in_quarter) =
                self.midi_header.division
            else {
                println!("unhandled division");
                return;
            };
            ticks_in_quarter as usize * 4
        };
        let mut track_positions = vec![0; self.tracks.len()];
        let mut current_tick = 0;
        let mut next_beat_tick = 0;
        let mut actual_time: f64 = 0.0;
        let mut bpm = 120.0;
        let mut beat = 0;

        let mut beats_paint_cache = vec![];
        let mut bars: Vec<CachedBar> = vec![];

        let mut time_signature = FileTimeSignature {
            numerator: 4,
            denominator: 4,
            clocks_per_metronome_tick: 24,
            thirty_second_notes_per_24_clocks: 8,
        };

        loop {
            let mut done = true;
            for (i, track) in self.tracks.iter().enumerate() {
                let track = track.track_original().midi_track();
                loop {
                    let event_idx = track_positions[i];
                    if event_idx >= track.len() {
                        break;
                    }
                    done = false;

                    let track_event = &track.events()[event_idx];
                    let event_tick = self
                        .midi_header
                        .division
                        .beat_or_frame_to_tick(track_event.beat_or_frame)
                        as usize;
                    if current_tick >= event_tick {
                        track_positions[i] += 1;
                        if let midi_msg::MidiMsg::Meta { msg } = &track_event.event {
                            match msg {
                                Meta::TimeSignature(ts) => {
                                    time_signature = ts.clone();
                                    beat = 0;
                                    self.marker_events.push((current_tick as f64, msg.clone()));
                                }
                                Meta::Marker(_) | Meta::EndOfTrack => {
                                    self.marker_events.push((current_tick as f64, msg.clone()));
                                }
                                Meta::SetTempo(tempo) => {
                                    bpm = 60_000_000. / f64::from(*tempo);
                                    self.marker_events.push((current_tick as f64, msg.clone()));
                                }
                                _ => (),
                            }
                        };
                    } else {
                        break;
                    }
                }
            }

            if current_tick == next_beat_tick {
                if let Some(prev) = bars.last_mut() {
                    prev.end_time = actual_time as f32;
                    prev.end_position = current_tick as f32;
                }
                bars.push(CachedBar {
                    start_time: actual_time as f32,
                    start_position: current_tick as f32,
                    end_time: f32::INFINITY,
                    end_position: f32::INFINITY,
                });

                beats_paint_cache.push((
                    [
                        vec2(current_tick as f32, 0.0),
                        vec2(current_tick as f32, 1.0),
                    ],
                    beat == 0,
                ));
                next_beat_tick += ticks_in_whole / time_signature.denominator as usize;
                beat += 1;
                beat %= time_signature.denominator;
            }

            current_tick += 1;
            actual_time += self.tick_duration(bpm);
            if done {
                break;
            }
        }

        self.beats_paint_cache = beats_paint_cache;
        self.cached_bars = bars;
    }

    fn tick_duration(&self, bpm: f64) -> f64 {
        match self.midi_header.division {
            Division::TicksPerQuarterNote(ticks) => 60. / bpm / f64::from(ticks),
            Division::TimeCode {
                frames_per_second,
                ticks_per_frame,
            } => {
                let fps = match frames_per_second {
                    TimeCodeType::FPS24 => 24.,
                    TimeCodeType::FPS25 => 25.,
                    TimeCodeType::DF30 | TimeCodeType::NDF30 => 30.,
                };
                1. / fps / f64::from(ticks_per_frame)
            }
        }
    }
}
