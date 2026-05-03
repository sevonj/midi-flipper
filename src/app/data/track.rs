// SPDX-License-Identifier: AGPL-3.0-or-later

pub(super) mod midi_data;

use std::time::Duration;

use crate::app::data::FlipSettings;
use crate::crustysynth::MidiRegion;
use midi_data::TrackMidiData;

#[derive(Debug)]
pub struct SessionTrack {
    name: Option<String>,

    track_original: TrackMidiData,
    track_flipped: TrackMidiData,

    settings: FlipSettings,

    changed: bool,
}

impl SessionTrack {
    pub fn from_track(
        midi_region: MidiRegion,
        global_transpose: i32,
        global_flip_bend: bool,
    ) -> Self {
        let settings = FlipSettings::new(global_transpose, global_flip_bend);
        let track_original = TrackMidiData::new(midi_region);

        let name = track_original.find_name();

        let track_flipped = track_original.clone().flipped(&settings);

        Self {
            name,
            track_original,
            track_flipped,
            settings,
            changed: false,
        }
    }

    pub(super) fn clear_changed(&mut self) -> bool {
        let changed = self.changed;
        self.changed = false;
        changed
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn length(&self) -> Duration {
        self.track_original().duration()
    }

    pub fn length_in_ticks(&self) -> u32 {
        self.track_original().length_in_ticks()
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

    pub(super) fn set_global_transpose(&mut self, global_transpose: i32) {
        self.settings.global_transpose = global_transpose;
        self.reflip();
    }

    pub(super) fn set_flip_bend(&mut self, flip_bend: bool) {
        self.settings.global_flip_bend = flip_bend;
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

    fn reflip(&mut self) {
        self.track_flipped = self.track_original.clone().flipped(&self.settings);
        self.changed = true;
    }
}
