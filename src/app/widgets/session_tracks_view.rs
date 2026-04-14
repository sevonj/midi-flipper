use egui::CentralPanel;
use egui::ScrollArea;
use egui::Vec2b;
use egui::Widget;
use egui_extras::Column;
use egui_extras::TableBuilder;

use crate::app::session::Session;
use crate::app::session::SessionTrack;

pub struct SessionTracksView<'a> {
    session: &'a mut Session,
}

impl<'a> SessionTracksView<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session }
    }
}

impl Widget for SessionTracksView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        CentralPanel::default()
            .show_inside(ui, |ui| {
                ScrollArea::new(Vec2b::new(false, true)).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(self.session.name());

                        ui.heading("Tracks");
                        let tablebuilder = TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::auto())
                            .column(Column::auto())
                            .column(Column::remainder())
                            .header(16.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Flip");
                                });
                                header.col(|ui| {
                                    ui.strong("Track No.");
                                });
                                header.col(|ui| {
                                    ui.strong("Name");
                                });
                            });

                        tablebuilder.body(|body| {
                            body.rows(30.0, self.session.tracks().len(), |mut row| {
                                let idx = row.index();
                                let track = &mut self.session.tracks_mut()[idx];
                                row.col(|ui| {
                                    ui.checkbox(track.flip_mut(), "").on_hover_text("Flip me?");
                                });
                                row.col(|ui| {
                                    ui.label(format!("Track {idx}"));
                                });
                                row.col(|ui| {
                                    track_name_label(ui, track);
                                });
                            });
                        });
                    });
                });
            })
            .response
    }
}

fn track_name_label(ui: &mut egui::Ui, track: &mut SessionTrack) {
    if !track.is_midi() {
        ui.disable();
        ui.label("unknown track type");
    } else if let Some(name) = track.name() {
        ui.label(name);
    } else {
        ui.weak("[unnamed track]");
    }
}
