use egui::Key;
use egui::KeyboardShortcut as SCut;
use egui::Modifiers;
use egui::Ui;

use crate::MidiFlipperApp;

const COMMAND: Modifiers = Modifiers::COMMAND;

pub const SHORTCUT_FILE_OPEN: SCut = SCut::new(COMMAND, Key::O);
pub const SHORTCUT_FILE_SAVE: SCut = SCut::new(COMMAND, Key::S);
pub const SHORTCUT_FILE_CLOSE: SCut = SCut::new(COMMAND, Key::W);
pub const SHORTCUT_QUIT: SCut = SCut::new(COMMAND, Key::Q);

impl MidiFlipperApp {
    pub(crate) fn consume_shortcuts(&mut self, ui: &mut Ui) {
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
    }
}
