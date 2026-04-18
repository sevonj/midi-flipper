use std::path::PathBuf;

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

    center_note: u8,
    flip_bend: bool,
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

        Ok(Self {
            name,
            midi_header: midi_file.header,
            length,
            tracks,
            center_note,
            flip_bend,
        })
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn tracks(&self) -> &[SessionTrack] {
        &self.tracks
    }

    pub fn tracks_mut(&mut self) -> &mut [SessionTrack] {
        &mut self.tracks
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
}
