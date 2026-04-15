use egui::Button;
use egui::MenuBar;
use egui::Panel;
use egui::Ui;

use crate::MidiFlipperApp;
use crate::app::shortcuts::SHORTCUT_FILE_CLOSE;
use crate::app::shortcuts::SHORTCUT_FILE_EXPORT;
use crate::app::shortcuts::SHORTCUT_FILE_OPEN;
use crate::app::shortcuts::SHORTCUT_QUIT;

impl MidiFlipperApp {
    pub(crate) fn top_panel(&mut self, ui: &mut Ui) -> egui::Response {
        Panel::top("top_panel")
            .show_inside(ui, |ui| {
                MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui
                            .add(
                                Button::new("Open")
                                    .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_FILE_OPEN)),
                            )
                            .clicked()
                        {
                            self.prompt_open_file();
                        }

                        if ui
                            .add(
                                Button::new("Export")
                                    .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_FILE_EXPORT)),
                            )
                            .clicked()
                        {
                            self.export_midi();
                        }

                        if ui
                            .add(
                                Button::new("Close")
                                    .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_FILE_CLOSE)),
                            )
                            .clicked()
                        {
                            self.close_session();
                        }

                        ui.separator();

                        if ui
                            .add(
                                Button::new("Quit")
                                    .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_QUIT)),
                            )
                            .clicked()
                        {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                })
            })
            .response
    }
}
