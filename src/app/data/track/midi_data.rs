// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Duration;

use midi_msg::ChannelVoiceMsg;
use midi_msg::Meta;
use midi_msg::MidiMsg;

use crate::app::data::FlipSettings;
use crate::app::data::PaintableNote;
use crate::crustysynth::MidiRegion;

#[derive(Debug, Clone)]
pub struct TrackMidiData {
    midi_region: MidiRegion,
    paint_cache: Vec<PaintableNote>,
}

impl TrackMidiData {
    pub fn new(midi_region: MidiRegion) -> Self {
        let mut this = Self {
            midi_region,
            paint_cache: vec![],
        };
        this.regenerate_paint_cache();
        this
    }

    pub fn flipped(mut self, settings: &FlipSettings) -> Self {
        self.flip(settings);
        self
    }

    pub fn duration(&self) -> Duration {
        self.midi_region
            .events()
            .last()
            .map(|event| event.time)
            .unwrap_or(Duration::ZERO)
    }

    pub fn length_in_ticks(&self) -> u32 {
        self.midi_region
            .events()
            .last()
            .map(|event| event.tick)
            .unwrap_or(0)
    }

    pub(crate) fn midi_region(&self) -> &MidiRegion {
        &self.midi_region
    }

    pub fn paint_cache(&self) -> &[PaintableNote] {
        &self.paint_cache
    }

    pub fn find_name(&self) -> Option<String> {
        for event in self.midi_region.events() {
            let MidiMsg::Meta { msg } = &event.msg else {
                continue;
            };
            if let Meta::TrackName(name) = msg {
                return Some(name.to_string());
            }
        }
        None
    }

    pub fn flip(&mut self, settings: &FlipSettings) {
        for event in self.midi_region.events_mut() {
            let (channel, msg) = match &mut event.msg {
                midi_msg::MidiMsg::ChannelVoice { channel, msg, .. } => (channel, msg),
                midi_msg::MidiMsg::RunningChannelVoice { channel, msg } => (channel, msg),
                _ => continue,
            };

            if settings.ignore_ch10 && *channel == midi_msg::Channel::Ch10 {
                continue;
            }

            let note = match msg {
                midi_msg::ChannelVoiceMsg::NoteOn { note, .. } => note,
                midi_msg::ChannelVoiceMsg::NoteOff { note, .. } => note,
                midi_msg::ChannelVoiceMsg::HighResNoteOn { note, .. } => note,
                midi_msg::ChannelVoiceMsg::HighResNoteOff { note, .. } => note,
                midi_msg::ChannelVoiceMsg::PolyPressure { note, .. } => note,
                midi_msg::ChannelVoiceMsg::PitchBend { bend } => {
                    if settings.global_flip_bend {
                        *bend = u16::MAX - *bend;
                    }
                    continue;
                }
                _ => continue,
            };

            let mut mapped = 127 - *note as i32;
            mapped += settings.global_transpose + settings.transpose;

            while mapped > 127 {
                mapped -= 12;
            }
            while mapped < 0 {
                mapped += 12;
            }

            *note = mapped as u8;
        }
        self.regenerate_paint_cache();
    }

    fn regenerate_paint_cache(&mut self) {
        self.paint_cache.clear();

        let mut open_notes = [None::<f32>; 128];

        for event in self.midi_region.events() {
            let msg = match event.msg {
                MidiMsg::ChannelVoice { msg, .. } => msg,
                MidiMsg::RunningChannelVoice { msg, .. } => msg,
                _ => continue,
            };

            let (note, note_on) = match msg {
                // Zero-vel NoteOn is considered a NoteOff
                ChannelVoiceMsg::NoteOn { note, velocity } => (note, velocity > 0),
                ChannelVoiceMsg::HighResNoteOn { note, velocity } => (note, velocity > 0),
                ChannelVoiceMsg::NoteOff { note, .. } => (note, false),
                ChannelVoiceMsg::HighResNoteOff { note, .. } => (note, false),
                _ => continue,
            };

            if note > 127 {
                continue;
            }

            let time = event.tick as f32;
            if note_on {
                if open_notes[note as usize].is_none() {
                    open_notes[note as usize] = Some(time);
                }
            } else {
                // note_off
                let Some(start) = open_notes[note as usize].take() else {
                    continue;
                };
                self.paint_cache.push(PaintableNote::new(note, start, time));
            }
        }

        // Check for unclosed notes
        let end = self.length_in_ticks() as f32;
        for (note, start) in open_notes.into_iter().enumerate() {
            if let Some(start) = start {
                self.paint_cache
                    .push(PaintableNote::new(note as u8, start, end));
            }
        }
    }
}
