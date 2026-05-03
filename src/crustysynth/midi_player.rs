// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::Duration;

use midi_msg::MidiFile;
use rustysynth::SoundFont;
use rustysynth::Synthesizer;
use rustysynth::SynthesizerSettings;

use super::midi_sequencer::MidiSequencer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioChannel {
    L,
    R,
}

pub struct MidiPlayer {
    synthesizer: Synthesizer,
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
        let mut synthesizer =
            Synthesizer::new(sf, &settings).expect("Could not create synthesizer");
        synthesizer.set_master_volume(1.0);
        let sequencer = MidiSequencer::new(midi_file);

        let sample_duration = Duration::from_secs_f64(1. / f64::from(Self::SAMPLE_RATE));
        Self {
            synthesizer,
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

    #[allow(dead_code)]
    pub fn sample_rate(&self) -> i32 {
        self.synthesizer.get_sample_rate()
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

                self.sequencer
                    .update_events(&mut self.synthesizer, self.sample_duration);
                let mut left = [0.0];
                let mut right = [0.0];
                self.synthesizer.render(&mut left, &mut right);
                self.cached_sample = right[0] * Self::GAIN;

                Some(left[0] * Self::GAIN)
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
        let samples_left = time_left.as_secs_f64() * f64::from(self.synthesizer.get_sample_rate());
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
        self.sequencer.seek_to(&mut self.synthesizer, pos);
        Ok(())
    }
}
