use egui::CentralPanel;
use egui::Panel;
use egui::Slider;
use egui::Widget;

use crate::app::data::Session;
use crate::app::widgets::TracksView;
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
                Panel::top("session_controls")
                    .show_separator_line(true)
                    .show_inside(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(self.session.name());

                            ui.horizontal(|ui| {
                                let mut center_note = self.session.center_note();
                                if ui
                                    .add(Slider::new(&mut center_note, 21..=127).custom_formatter(
                                        |v, _| util::note_name(v as u8).to_string(),
                                    ))
                                    .changed()
                                {
                                    self.session.set_center_note(center_note);
                                }
                                if ui.button("Reset").clicked() {
                                    self.session.reset_center_note();
                                }
                            });

                            let mut flip_bend = self.session.flip_bend();
                            if ui.checkbox(&mut flip_bend, "Flip pitch bend").changed() {
                                self.session.set_flip_bend(flip_bend);
                            }

                            ui.add_space(4.);
                        });
                    });
                ui.add_space(4.);
                ui.add(TracksView::new(self.session));
            })
            .response
    }
}
