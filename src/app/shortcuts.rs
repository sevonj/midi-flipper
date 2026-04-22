// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Key;
use egui::KeyboardShortcut as SCut;
use egui::Modifiers;
use egui::Ui;

use crate::MidiFlipperApp;

const COMMAND: Modifiers = Modifiers::COMMAND;
const NONE: Modifiers = Modifiers::NONE;

pub const SHORTCUT_FILE_OPEN: SCut = SCut::new(COMMAND, Key::O);
pub const SHORTCUT_FILE_SAVE: SCut = SCut::new(COMMAND, Key::S);
pub const SHORTCUT_FILE_CLOSE: SCut = SCut::new(COMMAND, Key::W);
pub const SHORTCUT_QUIT: SCut = SCut::new(COMMAND, Key::Q);

pub const SHORTCUT_VP_ZOOM_RESET: SCut = SCut::new(COMMAND, Key::Num0);
pub const SHORTCUT_VP_ZOOM_H_IN: SCut = SCut::new(COMMAND, Key::Plus);
pub const SHORTCUT_VP_ZOOM_H_OUT: SCut = SCut::new(COMMAND, Key::Minus);
pub const SHORTCUT_VP_ZOOM_V_IN: SCut = SCut::new(COMMAND, Key::PageUp);
pub const SHORTCUT_VP_ZOOM_V_OUT: SCut = SCut::new(COMMAND, Key::PageDown);
pub const SHORTCUT_VP_START: SCut = SCut::new(NONE, Key::W);
pub const SHORTCUT_VP_START_ALT: SCut = SCut::new(NONE, Key::Home);

pub const SHORTCUT_PLAYBACK_PLAYSTOP: SCut = SCut::new(NONE, Key::Space);
pub const SHORTCUT_PLAYBACK_PAUSE: SCut = SCut::new(COMMAND, Key::Space);

impl MidiFlipperApp {
    pub(crate) fn consume_shortcuts(&mut self, ui: &mut Ui) {
        // --- File

        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_FILE_OPEN)) {
            self.prompt_open_file();
        }

        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_FILE_SAVE)) {
            self.prompt_save_file();
        }

        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_FILE_CLOSE)) {
            self.close_session();
        }

        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_QUIT)) {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // --- Playback

        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_PLAYBACK_PLAYSTOP)) {
            if !self.session.is_playing() {
                self.session.play();
            } else {
                self.session.stop();
            }
        }

        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_PLAYBACK_PAUSE))
            && self.session.is_playback_in_progress()
        {
            if self.session.is_playing() {
                self.session.pause();
            } else {
                self.session.play();
            }
        }
    }
}
