use std::path::PathBuf;

use midi_msg::MidiFile;

use crate::MidiFlipperError;

#[derive(Debug)]
pub(crate) struct Session {
    name: String,
    midi_header: midi_msg::Header,
    tracks: Vec<SessionTrack>,
    flipped_midi: Option<Box<MidiFile>>,
}

#[derive(Debug)]
pub(crate) struct SessionTrack {
    name: Option<String>,
    track: midi_msg::Track,
    flip: bool,
}

impl SessionTrack {
    pub fn from_track(track: midi_msg::Track) -> Self {
        let name = Self::find_name(&track);
        Self {
            name,
            track,
            flip: true,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn flip(&self) -> bool {
        self.flip
    }

    pub fn flip_mut(&mut self) -> &mut bool {
        &mut self.flip
    }

    pub fn is_midi(&self) -> bool {
        matches!(self.track, midi_msg::Track::Midi(_))
    }

    fn find_name(track: &midi_msg::Track) -> Option<String> {
        let midi_msg::Track::Midi(events) = track else {
            return None;
        };

        for track_event in events {
            let midi_msg::MidiMsg::Meta { msg } = &track_event.event else {
                continue;
            };
            let midi_msg::Meta::TrackName(name) = msg else {
                continue;
            };
            return Some(name.to_owned());
        }
        None
    }
}

const FLIP_ORIGIN: i32 = 60; // 60 is C4

impl Session {
    pub fn new(name: String, midi_file: MidiFile) -> Self {
        let mut midi_tracks = Vec::with_capacity(midi_file.tracks.len());
        for track in midi_file.tracks {
            midi_tracks.push(SessionTrack::from_track(track));
        }
        Self {
            name,
            midi_header: midi_file.header,
            tracks: midi_tracks,
            flipped_midi: None,
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

    pub fn tracks(&self) -> &[SessionTrack] {
        &self.tracks
    }

    pub fn tracks_mut(&mut self) -> &mut [SessionTrack] {
        &mut self.tracks
    }

    pub fn flipped_midi(&self) -> Option<&MidiFile> {
        self.flipped_midi.as_deref()
    }

    pub fn flip(&mut self) -> Result<(), MidiFlipperError> {
        let mut midi_tracks = Vec::with_capacity(self.tracks.len());
        for session_track in &self.tracks {
            let mut track = session_track.track.clone();
            if session_track.flip() {
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
            let msg = match &mut track_event.event {
                midi_msg::MidiMsg::ChannelVoice { msg, .. } => msg,
                midi_msg::MidiMsg::RunningChannelVoice { msg, .. } => msg,
                _ => continue,
            };

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

            let mut mapped = FLIP_ORIGIN + (FLIP_ORIGIN - *note as i32);

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
