// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Button;
use egui::Frame;
use egui::MenuBar;
use egui::Panel;
use egui::Ui;
use egui::vec2;

use crate::MidiFlipperApp;
use crate::app::shortcuts::SHORTCUT_FILE_CLOSE;
use crate::app::shortcuts::SHORTCUT_FILE_OPEN;
use crate::app::shortcuts::SHORTCUT_FILE_SAVE;
use crate::app::shortcuts::SHORTCUT_QUIT;

impl MidiFlipperApp {
    pub(crate) fn menu_bar(&mut self, ui: &mut Ui) -> egui::Response {
        Panel::top("menu_bar")
            .resizable(false)
            .show_separator_line(false)
            .frame(
                Frame::default()
                    .inner_margin(vec2(8., 2.))
                    .fill(ui.ctx().global_style().visuals.widgets.open.weak_bg_fill),
            )
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
                            .add_enabled(
                                self.can_save(),
                                Button::new("Save")
                                    .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_FILE_SAVE)),
                            )
                            .clicked()
                        {
                            self.prompt_save_file();
                        }

                        if ui
                            .add_enabled(
                                self.is_session_open(),
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
