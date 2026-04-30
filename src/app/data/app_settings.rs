// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use rustysynth::SoundFont;

pub struct AppSettings {
    pub custom_soundfont: Option<Arc<SoundFont>>,
    pub master_volume: f32,
    pub border: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            custom_soundfont: None,
            master_volume: 1.0,
            border: false,
        }
    }
}
