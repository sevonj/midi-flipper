use std::path::PathBuf;

use midi_msg::MidiFile;

use crate::MidiFlipperError;

#[derive(Debug)]
pub(crate) struct Session {
    name: String,
    midi_file: Box<MidiFile>,
    flipped_midi: Option<Box<MidiFile>>,
}

const FLIP_ORIGIN: i32 = 60; // 60 is C4

impl Session {
    pub fn new(name: String, midi_file: MidiFile) -> Self {
        Self {
            name,
            midi_file: Box::new(midi_file),
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

    pub fn flipped_midi(&self) -> Option<&MidiFile> {
        self.flipped_midi.as_deref()
    }

    pub fn flip(&mut self) -> Result<(), MidiFlipperError> {
        let mut clone = self.midi_file.clone();

        for track in &mut clone.tracks {
            self.flip_track(track);
        }

        let validate = MidiFile::from_midi(&clone.to_midi());
        if validate.is_err() {
            return Err(MidiFlipperError::OutputValidationFailed);
        }

        self.flipped_midi = Some(clone);
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
