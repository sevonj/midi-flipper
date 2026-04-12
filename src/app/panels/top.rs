use egui::MenuBar;
use egui::Panel;
use egui::Ui;

use crate::MidiFlipperApp;

impl MidiFlipperApp {
    pub(crate) fn top_panel(&mut self, ui: &mut Ui) -> egui::Response {
        Panel::top("top_panel")
            .show_inside(ui, |ui| {
                MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            if let Some(file_path) = self.pick_midi_file() {
                                self.try_open_file(file_path);
                            }
                        }
                        if ui.button("Export").clicked() {
                            self.export_midi();
                        }
                        if ui.button("Close").clicked() {
                            self.close_session();
                        }
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                })
            })
            .response
    }
}
