pub(super) mod midi_data;

use midi_msg::Track as MidiTrack;

use crate::app::data::FlipSettings;
use midi_data::TrackMidiData;

#[derive(Debug)]
pub struct SessionTrack {
    name: Option<String>,
    length: f32,

    track_original: TrackMidiData,
    track_flipped: TrackMidiData,

    settings: FlipSettings,
}

impl SessionTrack {
    pub fn from_track(midi_track: MidiTrack, center_note: u8) -> Self {
        let settings = FlipSettings::new(center_note);
        let track_original = TrackMidiData::new(midi_track);

        let name = track_original.find_name();
        let length = track_original.length();

        let track_flipped = track_original.clone().flipped(&settings);

        Self {
            name,
            length,
            track_original,
            track_flipped,
            settings,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn track_original(&self) -> &TrackMidiData {
        &self.track_original
    }

    pub fn track_flipped(&self) -> &TrackMidiData {
        &self.track_flipped
    }

    pub fn flip_enabled(&self) -> bool {
        self.settings.enabled
    }

    pub fn set_flip_enabled(&mut self, flip_enabled: bool) {
        self.settings.enabled = flip_enabled;
        self.reflip();
    }

    pub(super) fn set_center_note(&mut self, center_note: u8) {
        self.settings.global_center = center_note;
        self.reflip();
    }

    pub fn transposition(&self) -> i32 {
        self.settings.transpose
    }

    pub fn set_transposition(&mut self, transposition: i32) {
        self.settings.transpose = transposition.clamp(-127, 127);
        self.reflip();
    }

    pub fn ignore_ch10(&self) -> bool {
        self.settings.ignore_ch10
    }

    pub fn set_ignore_ch10(&mut self, ignore_ch10: bool) {
        self.settings.ignore_ch10 = ignore_ch10;
        self.reflip();
    }

    pub fn is_midi(&self) -> bool {
        self.track_original.is_midi()
    }

    fn reflip(&mut self) {
        self.track_flipped = self.track_original.clone().flipped(&self.settings);
    }
}
