use std::path::PathBuf;

use midi_msg::Channel;
use midi_msg::MidiFile;

use crate::MidiFlipperError;
use crate::app::data::SessionTrack;

const MIDDLE_C: u8 = 60; // 60 is C4

#[derive(Debug)]
pub struct Session {
    name: String,
    length: f32,
    midi_header: midi_msg::Header,
    tracks: Vec<SessionTrack>,
    flipped_midi: Option<Box<MidiFile>>,
    ignore_ch10: bool,
    flip_center: u8,
}

impl Session {
    pub fn new(name: String, midi_file: MidiFile) -> Self {
        let mut tracks = Vec::with_capacity(midi_file.tracks.len());
        let mut length = 0.0;
        for track in midi_file.tracks {
            let session_track = SessionTrack::from_track(track);
            if session_track.length() > length {
                length = session_track.length();
            }
            tracks.push(session_track);
        }

        Self {
            name,
            midi_header: midi_file.header,
            length,
            tracks,
            flipped_midi: None,
            ignore_ch10: true,
            flip_center: MIDDLE_C,
        }
    }

    pub fn from_file(file_path: PathBuf) -> Result<Self, MidiFlipperError> {
        let name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let bytes = std::fs::read(file_path)?;
        let midi_file = MidiFile::from_midi(&bytes)?;

        Ok(Self::new(name, midi_file))
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

    pub fn flipped_midi(&self) -> Option<&MidiFile> {
        self.flipped_midi.as_deref()
    }

    pub fn ignore_ch10(&self) -> bool {
        self.ignore_ch10
    }

    pub fn ignore_ch10_mut(&mut self) -> &mut bool {
        &mut self.ignore_ch10
    }

    pub fn flip_center(&self) -> u8 {
        self.flip_center
    }

    pub fn flip_center_mut(&mut self) -> &mut u8 {
        &mut self.flip_center
    }

    pub fn reset_flip_center(&mut self) {
        self.flip_center = MIDDLE_C;
    }

    pub fn flip(&mut self) -> Result<(), MidiFlipperError> {
        let mut midi_tracks = Vec::with_capacity(self.tracks.len());
        for session_track in &self.tracks {
            let mut track = session_track.track().clone();
            if session_track.flip_enabled() {
                self.flip_track(&mut track);
            }
            midi_tracks.push(track);
        }

        let flipped = MidiFile {
            header: self.midi_header.clone(),
            tracks: midi_tracks,
        };

        let validate = MidiFile::from_midi(&flipped.to_midi());
        if validate.is_err() {
            return Err(MidiFlipperError::OutputValidationFailed);
        }

        self.flipped_midi = Some(Box::new(flipped));

        Ok(())
    }

    fn flip_track(&self, track: &mut midi_msg::Track) {
        let midi_msg::Track::Midi(track_events) = track else {
            return;
        };

        for track_event in track_events {
            let (channel, msg) = match &mut track_event.event {
                midi_msg::MidiMsg::ChannelVoice { channel, msg, .. } => (channel, msg),
                midi_msg::MidiMsg::RunningChannelVoice { channel, msg } => (channel, msg),
                _ => continue,
            };

            if self.ignore_ch10 && *channel == Channel::Ch10 {
                continue;
            }

            let note = match msg {
                midi_msg::ChannelVoiceMsg::NoteOn { note, .. } => note,
                midi_msg::ChannelVoiceMsg::NoteOff { note, .. } => note,
                midi_msg::ChannelVoiceMsg::HighResNoteOn { note, .. } => note,
                midi_msg::ChannelVoiceMsg::HighResNoteOff { note, .. } => note,
                midi_msg::ChannelVoiceMsg::PolyPressure { note, .. } => note,
                // midi_msg::ChannelVoiceMsg::PitchBend { bend } => {
                //    // TODO: flip bend
                //    continue;
                //}
                _ => continue,
            };

            let flip_center = self.flip_center as i32;
            let mut mapped = flip_center + (flip_center - *note as i32);

            while mapped > 127 {
                mapped -= 12;
            }
            while mapped < 0 {
                mapped += 12;
            }

            *note = mapped as u8;
        }
    }
}
