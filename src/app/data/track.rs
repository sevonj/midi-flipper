use midi_msg::ChannelVoiceMsg;
use midi_msg::MidiMsg;
use midi_msg::TrackEvent;

use crate::app::data::PaintableNote;

#[derive(Debug)]
pub struct SessionTrack {
    name: Option<String>,
    track_original: midi_msg::Track,
    flip_enabled: bool,

    paint_cache_og: Vec<PaintableNote>,
}

impl SessionTrack {
    pub fn from_track(track: midi_msg::Track) -> Self {
        let name = Self::find_name(&track);

        let mut paint_cache_og = vec![];
        regenerate_paint_cache(&mut paint_cache_og, &track);

        Self {
            name,
            track_original: track,
            flip_enabled: true,
            paint_cache_og,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn track(&self) -> &midi_msg::Track {
        &self.track_original
    }

    pub fn flip_enabled(&self) -> bool {
        self.flip_enabled
    }

    pub fn flip_enabled_mut(&mut self) -> &mut bool {
        &mut self.flip_enabled
    }

    pub fn note_paint_cache(&self) -> &[PaintableNote] {
        &self.paint_cache_og
    }

    pub fn is_midi(&self) -> bool {
        matches!(self.track_original, midi_msg::Track::Midi(_))
    }

    pub fn events(&self) -> Option<&Vec<TrackEvent>> {
        let midi_msg::Track::Midi(track_events) = &self.track_original else {
            return None;
        };
        Some(&track_events)
    }

    fn find_name(track: &midi_msg::Track) -> Option<String> {
        let midi_msg::Track::Midi(events) = track else {
            return None;
        };

        for track_event in events {
            let MidiMsg::Meta { msg } = &track_event.event else {
                continue;
            };
            let midi_msg::Meta::TrackName(name) = msg else {
                continue;
            };
            return Some(name.to_owned());
        }
        None
    }
}

fn regenerate_paint_cache(cache: &mut Vec<PaintableNote>, track: &midi_msg::Track) {
    cache.clear();

    let midi_msg::Track::Midi(track_events) = &track else {
        return;
    };

    let mut open_notes = [None::<f32>; 128];
    let mut time = 0.0;

    for event in track_events {
        time += event.beat_or_frame;

        let msg = match event.event {
            MidiMsg::ChannelVoice { msg, .. } => msg,
            MidiMsg::RunningChannelVoice { msg, .. } => msg,
            _ => continue,
        };

        let (note, note_on) = match msg {
            ChannelVoiceMsg::NoteOn { note, .. } | ChannelVoiceMsg::HighResNoteOn { note, .. } => {
                (note, true)
            }
            ChannelVoiceMsg::NoteOff { note, .. }
            | ChannelVoiceMsg::HighResNoteOff { note, .. } => (note, false),
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
            cache.push(PaintableNote::new(note, start, time));
        }
    }

    // Check for unclosed notes
    for (note, start) in open_notes.into_iter().enumerate() {
        if let Some(start) = start {
            cache.push(PaintableNote::new(note as u8, start, time));
        }
    }
}
