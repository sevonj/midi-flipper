use egui::CentralPanel;
use egui::ScrollArea;
use egui::Vec2b;
use egui::Widget;

use crate::app::session::Session;

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

                        ui.checkbox(self.session.ignore_ch10_mut(), "Skip Ch. 10 events (drums)");

                        if self.session.flipped_midi().is_none() {
                            ui.label("not flipped");
                        } else {
                            ui.label("is flipped");
                        }

                        if ui.button("Flip").clicked()
                            && let Err(_e) = self.session.flip()
                        {
                            //
                        };
                    });
                });
            })
            .response
    }
}
