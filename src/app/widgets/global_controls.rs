// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Slider;
use egui::Widget;

use crate::app::data::Session;

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
        if self.session.is_placeholder() {
            ui.disable();
        }

        ui.horizontal(|ui| {
            let mut global_transpose = self.session.global_transpose();
            if ui
                .add(Slider::new(&mut global_transpose, -127..=127).smart_aim(false))
                .on_hover_text("Global Transposition")
                .changed()
            {
                self.session.set_global_transpose(global_transpose);
            }
            if ui.button("Reset").clicked() {
                self.session.reset_global_transpose();
            }
        });

        let mut flip_pitch_bend = self.session.flip_bend();
        if ui
            .checkbox(&mut flip_pitch_bend, "Flip pitch bend")
            .changed()
        {
            self.session.set_flip_bend(flip_pitch_bend);
        }

        ui.add_space(4.);

        ui.response()
    }
}
