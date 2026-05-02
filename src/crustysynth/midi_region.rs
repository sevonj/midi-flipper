use std::time::Duration;

use midi_msg::MidiMsg;

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub time: Duration,
    pub msg: MidiMsg,
}

impl MidiEvent {
    pub fn new(time: Duration, msg: MidiMsg) -> Self {
        Self { time, msg }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MidiRegion {
    events: Vec<MidiEvent>,
}

impl MidiRegion {
    pub fn new(events: Vec<MidiEvent>) -> Self {
        Self { events }
    }

    pub fn events(&self) -> &[MidiEvent] {
        &self.events
    }

    pub fn duration(&self) -> Duration {
        self.events
            .last()
            .map(|last| last.time)
            .unwrap_or(Duration::ZERO)
    }
}
