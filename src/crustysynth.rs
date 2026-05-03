// SPDX-License-Identifier: AGPL-3.0-or-later

mod midi_player;
mod midi_region;
mod midi_sequencer;
mod midi_sink;

use midi_msg::MidiFile;
#[cfg(not(feature = "ci"))]
use rodio::MixerDeviceSink;
use rodio::Player;
use rustysynth::SoundFont;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(feature = "ci"))]
use midi_player::MidiPlayer;
use midi_sequencer::MidiSequencer;

const DEFAULT_SOUNDFONT: &[u8] = include_bytes!("../assets/__Florestan_Basic_GM_GS.sf2");

pub struct CrustySynth {
    soundfont: Arc<SoundFont>,
    midi_file: Option<(Arc<MidiFile>, Duration)>,
    #[cfg(not(feature = "ci"))]
    sink_handle: MixerDeviceSink,
    player: Option<Player>,
    volume: f32,
}

impl Default for CrustySynth {
    fn default() -> Self {
        #[cfg(not(feature = "ci"))]
        let sink_handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("Couldn't open sink");

        Self {
            soundfont: Self::default_soundfont(),
            midi_file: None,
            #[cfg(not(feature = "ci"))]
            sink_handle,
            player: None,
            volume: 1.0,
        }
    }
}

impl CrustySynth {
    pub fn default_soundfont() -> Arc<SoundFont> {
        Arc::new(SoundFont::new(&mut BufReader::new(DEFAULT_SOUNDFONT)).unwrap())
    }

    pub fn midi_file(&self) -> Option<&Arc<MidiFile>> {
        if let Some((midi_file, _)) = &self.midi_file {
            return Some(midi_file);
        }
        None
    }

    pub fn set_midi_file(&mut self, midi_file: Option<Arc<MidiFile>>) {
        self.stop();
        if let Some(midi_file) = midi_file {
            let duration = MidiSequencer::new(&midi_file).song_duration();
            self.midi_file = Some((midi_file, duration))
        } else {
            self.midi_file = None;
        }
    }

    // Swap on the fly, attempt to keep playback state
    pub fn swap_midi_file(&mut self, midi_file: Arc<MidiFile>) {
        #[cfg(not(feature = "ci"))]
        if let Some(old) = self.player.take() {
            let paused = old.is_paused();
            let pos = old.get_pos();

            let new = Player::connect_new(self.sink_handle.mixer());
            let source = MidiPlayer::new(&self.soundfont, &midi_file);
            new.append(source);
            let _ = new.try_seek(pos);

            new.play();
            if paused {
                new.pause();
            }
            self.player = Some(new)
        }
        let duration = MidiSequencer::new(&midi_file).song_duration();
        self.midi_file = Some((midi_file, duration));
    }

    #[allow(dead_code)]
    pub fn soundfont(&self) -> &Arc<SoundFont> {
        &self.soundfont
    }

    #[allow(dead_code)]
    pub fn set_soundfont(&mut self, soundfont: Arc<SoundFont>) {
        self.stop();
        self.soundfont = soundfont;
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(player) = &self.player {
            player.set_volume(volume);
        };
    }

    pub fn is_playing(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(|p| !p.is_paused() && !p.empty())
    }

    pub fn is_playback_in_progress(&self) -> bool {
        self.player.as_ref().is_some_and(|p| !p.empty())
    }

    pub fn position(&self) -> Duration {
        self.player
            .as_ref()
            .map(|p| p.get_pos())
            .unwrap_or(Duration::ZERO)
    }

    pub fn duration(&self) -> Duration {
        if let Some((_, duration)) = &self.midi_file {
            return *duration;
        }
        Duration::ZERO
    }

    pub fn start(&mut self) {
        #[cfg(not(feature = "ci"))]
        {
            self.stop();
            let Some(midi_file) = self.midi_file() else {
                return;
            };

            let player = Player::connect_new(self.sink_handle.mixer());
            player.set_volume(self.volume);
            let source = MidiPlayer::new(&self.soundfont, midi_file);
            player.append(source);
            player.play();
            self.player = Some(player)
        }
    }

    pub fn play(&mut self) {
        if !self.is_playback_in_progress() {
            self.start();
            return;
        }
        let Some(player) = &self.player else {
            self.start();
            return;
        };
        player.play();
    }

    pub fn pause(&mut self) {
        if let Some(player) = &self.player {
            player.pause();
        }
    }

    pub fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
    }

    pub fn seek_to(&mut self, time: f32) {
        if let Some(player) = &mut self.player {
            let _ = player.try_seek(Duration::from_secs_f32(time));
        }
    }
}
