// SPDX-License-Identifier: AGPL-3.0-or-later

mod midi_player;
mod midi_region;
mod midi_sequencer;
mod midi_sink;
mod vu_meter;

#[cfg(not(feature = "ci"))]
use rodio::MixerDeviceSink;
use rodio::Player;
use rustysynth::SoundFont;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(feature = "ci"))]
use midi_player::MidiPlayer;
pub use midi_region::MidiRegion;

use crate::crustysynth::vu_meter::StereoVUReceiver;

const DEFAULT_SOUNDFONT: &[u8] = include_bytes!("../assets/__Florestan_Basic_GM_GS.sf2");

pub struct CrustySynth {
    soundfont: Arc<SoundFont>,
    midi: Option<Arc<Vec<MidiRegion>>>,
    #[cfg(not(feature = "ci"))]
    sink_handle: MixerDeviceSink,
    player: Option<Player>,
    volume: f32,
    master_vu: StereoVUReceiver,
}

impl Default for CrustySynth {
    fn default() -> Self {
        #[cfg(not(feature = "ci"))]
        let sink_handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("Couldn't open sink");

        Self {
            soundfont: Self::default_soundfont(),
            midi: None,
            #[cfg(not(feature = "ci"))]
            sink_handle,
            player: None,
            volume: 1.0,
            master_vu: StereoVUReceiver::dummy(),
        }
    }
}

impl CrustySynth {
    pub fn default_soundfont() -> Arc<SoundFont> {
        Arc::new(SoundFont::new(&mut BufReader::new(DEFAULT_SOUNDFONT)).unwrap())
    }

    pub fn midi(&self) -> Option<&Arc<Vec<MidiRegion>>> {
        self.midi.as_ref()
    }

    pub fn set_midi(&mut self, midi: Option<Arc<Vec<MidiRegion>>>) {
        self.stop();
        self.midi = midi;
    }

    // Swap on the fly, attempt to keep playback state
    pub fn swap_midi(&mut self, midi: Arc<Vec<MidiRegion>>) {
        #[cfg(not(feature = "ci"))]
        if let Some(old) = self.player.take() {
            let paused = old.is_paused();
            let pos = old.get_pos();

            let new = Player::connect_new(self.sink_handle.mixer());
            let source = MidiPlayer::new(&self.soundfont, &midi);
            self.master_vu = source.master_vu_receiver();
            new.append(source);
            let _ = new.try_seek(pos);

            new.play();
            if paused {
                new.pause();
            }
            self.player = Some(new)
        }
        self.midi = Some(midi);
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

    pub fn master_vu(&self) -> &StereoVUReceiver {
        &self.master_vu
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
        self.midi
            .as_ref()
            .and_then(|tracks| tracks.iter().map(|region| region.duration()).max())
            .unwrap_or(Duration::ZERO)
    }

    pub fn start(&mut self) {
        #[cfg(not(feature = "ci"))]
        {
            self.stop();
            let Some(midi) = self.midi() else {
                return;
            };

            let player = Player::connect_new(self.sink_handle.mixer());
            player.set_volume(self.volume);
            let source = MidiPlayer::new(&self.soundfont, midi);
            self.master_vu = source.master_vu_receiver();
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
