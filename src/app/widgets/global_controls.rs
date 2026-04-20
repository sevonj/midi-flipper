// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Slider;
use egui::Widget;

use crate::app::data::Session;
use crate::util;

pub struct GlobalControls<'a> {
    session: &'a mut Session,
}

impl<'a> GlobalControls<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session }
    }
}

impl Widget for GlobalControls<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            if self.session.is_placeholder() {
                ui.disable();
            }

            let mut flip_pitch_bend = self.session.flip_bend();
            if ui
                .checkbox(&mut flip_pitch_bend, "Flip pitch bend")
                .changed()
            {
                self.session.set_flip_bend(flip_pitch_bend);
            }

            ui.horizontal(|ui| {
                let mut center_note = self.session.center_note();
                if ui
                    .add(
                        Slider::new(&mut center_note, 21..=127)
                            .custom_formatter(|v, _| util::note_name(v as u8).to_string()),
                    )
                    .changed()
                {
                    self.session.set_center_note(center_note);
                }
                if ui.button("Reset").clicked() {
                    self.session.reset_center_note();
                }
            });

            ui.add_space(4.);
        })
        .response
    }
}
