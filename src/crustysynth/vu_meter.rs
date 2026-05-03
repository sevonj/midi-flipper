// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::crustysynth::midi_player::MidiPlayer;

const SAMPLE_RATE: i32 = MidiPlayer::SAMPLE_RATE;
const RMS_WINDOW: u32 = SAMPLE_RATE as u32 / 60;

#[derive(Default)]
pub struct StereoVUSender {
    l: VUSender,
    r: VUSender,
}

impl StereoVUSender {
    pub fn receiver(&self) -> StereoVUReceiver {
        StereoVUReceiver {
            l: self.l.receiver(),
            r: self.r.receiver(),
        }
    }

    pub fn add_sample(&mut self, l: f32, r: f32) {
        self.l.add_sample(l);
        self.r.add_sample(r);
    }
}

pub struct StereoVUReceiver {
    l: VUReceiver,
    r: VUReceiver,
}

impl StereoVUReceiver {
    pub fn dummy() -> Self {
        Self {
            l: VUReceiver::dummy(),
            r: VUReceiver::dummy(),
        }
    }

    /// (left, right)
    pub fn rms(&self) -> (f32, f32) {
        (self.l.rms(), self.r.rms())
    }
}

#[derive(Default)]
pub struct VUSender {
    rms_sample_counter: u32,
    rms_acc: f32,
    rms_atomic: Arc<AtomicU32>,
}

impl VUSender {
    pub fn receiver(&self) -> VUReceiver {
        VUReceiver {
            rms_atomic: self.rms_atomic.clone(),
        }
    }

    pub fn add_sample(&mut self, sample: f32) {
        self.rms_acc += sample * sample;
        self.rms_sample_counter += 1;
        if self.rms_sample_counter == RMS_WINDOW {
            self.rms_sample_counter = 0;
            let rms = (self.rms_acc / RMS_WINDOW as f32).sqrt();
            self.rms_atomic.store(rms.to_bits(), Ordering::Relaxed);
            self.rms_acc = 0.0;
        }
    }
}

pub struct VUReceiver {
    rms_atomic: Arc<AtomicU32>,
}

impl VUReceiver {
    pub fn dummy() -> Self {
        Self {
            rms_atomic: Default::default(),
        }
    }

    pub fn rms(&self) -> f32 {
        f32::from_bits(self.rms_atomic.load(Ordering::Relaxed))
    }
}
