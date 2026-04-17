use egui::CentralPanel;
use egui::ScrollArea;
use egui::Slider;
use egui::Vec2b;
use egui::Widget;

use crate::app::data::Session;
use crate::util;

pub struct SessionView<'a> {
    session: &'a mut Session,
}

impl<'a> SessionView<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session }
    }
}

impl Widget for SessionView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        CentralPanel::default()
            .show_inside(ui, |ui| {
                ScrollArea::new(Vec2b::new(false, true)).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(self.session.name());

                        ui.horizontal(|ui| {
                            let mut center_note = self.session.center_note();
                            if ui
                                .add(
                                    Slider::new(&mut center_note, 21..=127).custom_formatter(
                                        |v, _| util::note_name(v as u8).to_string(),
                                    ),
                                )
                                .changed()
                            {
                                self.session.set_center_note(center_note);
                            }
                            if ui.button("Reset").clicked() {
                                self.session.reset_center_note();
                            }
                        });
                    });
                });
            })
            .response
    }
}
