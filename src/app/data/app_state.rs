// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::VecDeque, sync::Arc, time::Instant};

use egui_toast::Toasts;
use rustysynth::SoundFont;

use crate::app::{AppTab, ModalState};

pub struct AppState {
    pub custom_soundfont: Option<Arc<SoundFont>>,
    pub master_volume: f32,
    pub toasts: Toasts,
    pub log: VecDeque<String>,
    pub tab: AppTab,
    pub start: Instant,
    pub splash_done: bool,
    pub session_init: bool,
    pub modal_state: ModalState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            custom_soundfont: None,
            master_volume: 1.0,
            toasts: Default::default(),
            log: Default::default(),
            tab: Default::default(),
            start: Instant::now(),
            splash_done: false,
            session_init: false,
            modal_state: Default::default(),
        }
    }
}
