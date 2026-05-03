// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::Duration;

use midi_msg::Channel;
use midi_msg::ChannelVoiceMsg;
use midi_msg::MidiFile;
use midi_msg::MidiMsg;
use rustysynth::SoundFont;
use rustysynth::Synthesizer;
use rustysynth::SynthesizerSettings;

use crate::crustysynth::midi_sink::MidiSink;

use super::midi_sequencer::MidiSequencer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioChannel {
    L,
    R,
}

pub struct MidiPlayer {
    synths: Vec<Synthesizer>,
    sequencer: MidiSequencer,
    sample_duration: Duration,
    cached_sample: f32,
    next_ch: AudioChannel,
}

impl MidiPlayer {
    pub const SAMPLE_RATE: i32 = 44100;
    pub const GAIN: f32 = 0.25;

    pub fn new(sf: &Arc<SoundFont>, midi_file: &Arc<MidiFile>) -> Self {
        let settings = SynthesizerSettings::new(Self::SAMPLE_RATE);

        let num_tracks = midi_file.tracks.len();
        let synths = (0..num_tracks)
            .map(|_| {
                let mut synth =
                    Synthesizer::new(sf, &settings).expect("Could not create synthesizer");
                synth.set_master_volume(1.0);
                synth
            })
            .collect();

        let sequencer = MidiSequencer::new(midi_file);

        let sample_duration = Duration::from_secs_f64(1. / f64::from(Self::SAMPLE_RATE));
        Self {
            synths,
            sample_duration,
            sequencer,
            next_ch: AudioChannel::L,
            cached_sample: 0.,
        }
    }

    #[allow(dead_code)]
    pub const fn song_length(&self) -> Duration {
        self.sequencer.song_duration()
    }

    fn handle_events(&mut self) {
        while let Some((track, msg)) = self.sequencer.take_event() {
            match msg {
                MidiMsg::ChannelVoice { .. }
                | MidiMsg::RunningChannelVoice { .. }
                | MidiMsg::ChannelMode { .. }
                | MidiMsg::RunningChannelMode { .. } => {
                    self.synths[track].receive_midi(msg);
                }
                _ => (),
            }
        }
    }

    // avoids unwanted noteon events
    fn handle_events_seek(&mut self) {
        let mut tracks_notes = vec![[[None; 128]; 16]; self.synths.len()];
        while let Some((track, msg)) = self.sequencer.take_event() {
            if let Some((channel, note, velocity)) = is_event_note_on_off(msg) {
                tracks_notes[track][channel as usize][note as usize] = Some(velocity);
                continue;
            }

            match msg {
                MidiMsg::ChannelVoice { .. }
                | MidiMsg::RunningChannelVoice { .. }
                | MidiMsg::ChannelMode { .. }
                | MidiMsg::RunningChannelMode { .. } => {
                    self.synths[track].receive_midi(msg);
                }
                _ => continue,
            }
        }

        for (track, channels) in tracks_notes.iter().enumerate() {
            for (channel, notes) in channels.iter().enumerate() {
                for (note, vel_opt) in notes.iter().enumerate() {
                    if let Some(velocity) = vel_opt {
                        self.synths[track].receive_midi(&MidiMsg::ChannelVoice {
                            channel: Channel::from_u8(channel as u8),
                            msg: ChannelVoiceMsg::NoteOn {
                                note: note as u8,
                                velocity: *velocity,
                            },
                        });
                    }
                }
            }
        }
    }
}

impl Iterator for MidiPlayer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sequencer.end_of_sequence() {
            return None;
        }

        // The synth generates both channels simultaneously,
        // but Rodio polls samples one at a time, expecting interleaved channels

        match self.next_ch {
            AudioChannel::L => {
                self.next_ch = AudioChannel::R;

                self.sequencer.advance_time(self.sample_duration);
                self.handle_events();

                let mut left = 0.0;
                let mut right = 0.0;
                for synth in &mut self.synths {
                    let mut lbuf = [0.0];
                    let mut rbuf = [0.0];
                    synth.render(&mut lbuf, &mut rbuf);
                    left += lbuf[0];
                    right += rbuf[0];
                }
                self.cached_sample = right * Self::GAIN;
                Some(left * Self::GAIN)
            }
            AudioChannel::R => {
                self.next_ch = AudioChannel::L;
                Some(self.cached_sample)
            }
        }
    }
}

impl rodio::Source for MidiPlayer {
    fn current_span_len(&self) -> Option<usize> {
        let time_left = self
            .sequencer
            .song_duration()
            .saturating_sub(self.sequencer.song_position());
        let samples_left = time_left.as_secs_f64() * f64::from(Self::SAMPLE_RATE);
        Some(samples_left as usize)
    }

    fn channels(&self) -> rodio::ChannelCount {
        rodio::ChannelCount::new(2).unwrap()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        rodio::SampleRate::new(Self::SAMPLE_RATE as u32).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.sequencer.song_duration())
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.sequencer.seek_to(self.synths.as_mut_slice(), pos);
        self.handle_events_seek();
        Ok(())
    }
}

fn is_event_note_on_off(msg: &MidiMsg) -> Option<(Channel, u8, u8)> {
    match msg {
        MidiMsg::ChannelVoice { channel, msg } | MidiMsg::RunningChannelVoice { channel, msg } => {
            match msg {
                ChannelVoiceMsg::NoteOn { note, velocity } => Some((*channel, *note, *velocity)),
                ChannelVoiceMsg::HighResNoteOn { note, velocity } => {
                    Some((*channel, *note, (*velocity >> 8) as u8))
                }
                ChannelVoiceMsg::NoteOff { note, .. }
                | ChannelVoiceMsg::HighResNoteOff { note, .. } => Some((*channel, *note, 0)),
                _ => None,
            }
        }
        _ => None,
    }
}
