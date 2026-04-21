// SPDX-License-Identifier: AGPL-3.0-or-later

use midi_msg::ChannelVoiceMsg;
use midi_msg::Meta;
use midi_msg::MidiMsg;
use midi_msg::Track as MidiTrack;

use crate::app::data::FlipSettings;
use crate::app::data::PaintableNote;

#[derive(Debug, Clone)]
pub struct TrackMidiData {
    midi_track: MidiTrack,
    paint_cache: Vec<PaintableNote>,
}

impl TrackMidiData {
    pub fn new(midi_track: MidiTrack) -> Self {
        let mut this = Self {
            midi_track,
            paint_cache: vec![],
        };
        this.regenerate_paint_cache();
        this
    }

    pub fn flipped(mut self, settings: &FlipSettings) -> Self {
        self.flip(settings);
        self
    }

    pub(crate) fn midi_track(&self) -> &MidiTrack {
        &self.midi_track
    }

    pub fn paint_cache(&self) -> &[PaintableNote] {
        &self.paint_cache
    }

    pub(crate) fn is_midi(&self) -> bool {
        matches!(self.midi_track, MidiTrack::Midi(_))
    }

    pub fn find_meta(&self) -> (Option<String>, f32) {
        let MidiTrack::Midi(events) = &self.midi_track else {
            return (None, 0.0);
        };

        let mut time = 0.0;
        let mut found_name = None;

        for track_event in events {
            time += track_event.delta_time as f32;
            let MidiMsg::Meta { msg } = &track_event.event else {
                continue;
            };
            if let Meta::TrackName(name) = msg {
                found_name = Some(name.to_string());
            }
        }
        (found_name, time)
    }

    pub fn flip(&mut self, settings: &FlipSettings) {
        let MidiTrack::Midi(track_events) = &mut self.midi_track else {
            return;
        };

        for track_event in track_events {
            let (channel, msg) = match &mut track_event.event {
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

        let MidiTrack::Midi(track_events) = &self.midi_track else {
            return;
        };

        let mut open_notes = [None::<f32>; 128];
        let mut time = 0.0;

        for event in track_events {
            time += event.delta_time as f32;

            let msg = match event.event {
                MidiMsg::ChannelVoice { msg, .. } => msg,
                MidiMsg::RunningChannelVoice { msg, .. } => msg,
                _ => continue,
            };

            let (note, note_on) = match msg {
                ChannelVoiceMsg::NoteOn { note, velocity } => (note, velocity > 0),
                ChannelVoiceMsg::HighResNoteOn { note, velocity } => (note, velocity > 0),
                ChannelVoiceMsg::NoteOff { note, .. } => (note, false),
                ChannelVoiceMsg::HighResNoteOff { note, .. } => (note, false),
                _ => continue,
            };

            if note > 127 {
                continue;
            }

            if note_on {
                open_notes[note as usize] = Some(time);
            } else {
                // note_off
                let Some(start) = open_notes[note as usize].take() else {
                    continue;
                };
                self.paint_cache.push(PaintableNote::new(note, start, time));
            }
        }

        // Check for unclosed notes
        for (note, start) in open_notes.into_iter().enumerate() {
            if let Some(start) = start {
                self.paint_cache
                    .push(PaintableNote::new(note as u8, start, time));
            }
        }
    }
}
