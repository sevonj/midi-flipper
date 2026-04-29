// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;

#[derive(Debug)]
pub enum MidiFlipperError {
    Io(std::io::Error),
    MidiParse(midi_msg::MidiFileParseError),
    FontParse(rustysynth::SoundFontError),
}

impl From<std::io::Error> for MidiFlipperError {
    fn from(source: std::io::Error) -> Self {
        Self::Io(source)
    }
}

impl From<midi_msg::MidiFileParseError> for MidiFlipperError {
    fn from(source: midi_msg::MidiFileParseError) -> Self {
        Self::MidiParse(source)
    }
}

impl From<rustysynth::SoundFontError> for MidiFlipperError {
    fn from(source: rustysynth::SoundFontError) -> Self {
        Self::FontParse(source)
    }
}

impl std::fmt::Display for MidiFlipperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MidiFlipperError::*;

        match self {
            Io(_) => write!(f, "IO Error"),
            MidiParse(_) => write!(f, "Couldn't parse MIDI file"),
            FontParse(_) => write!(f, "Couldn't parse soundfont"),
        }
    }
}

impl Error for MidiFlipperError {}
