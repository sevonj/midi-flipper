// SPDX-License-Identifier: AGPL-3.0-or-later

use midi_msg::MidiMsg;
use rustysynth::Synthesizer;

pub trait MidiSink {
    fn receive_midi(&mut self, msg: &MidiMsg);
    fn reset(&mut self);
}

impl MidiSink for Synthesizer {
    fn receive_midi(&mut self, msg: &MidiMsg) {
        let raw = msg.to_midi();
        match raw.len() {
            2..=3 => send_raw_event(self, &raw),
            5 => {
                // Break a message that contains MSB and LSB in one into two
                // separate ones for rustysynth consumption.
                if let 0x62 | 0x64 = raw[1] {
                    send_raw_event(self, &[raw[0], raw[3], raw[4]]);
                    send_raw_event(self, &raw[0..3]);
                }
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        self.reset();
    }
}

fn send_raw_event(synth: &mut Synthesizer, raw: &[u8]) {
    let channel = raw[0] & 0x0f;
    let command = raw[0] & 0xf0;
    let data1;
    let data2;
    match raw.len() {
        2 => {
            data1 = raw[1];
            data2 = 0;
        }
        3 => {
            data1 = raw[1];
            data2 = raw[2];
        }
        _ => unreachable!(),
    }
    synth.process_midi_message(channel.into(), command.into(), data1.into(), data2.into());
}
