use egui::CentralPanel;
use egui::Widget;
use egui_extras::Column;
use egui_extras::TableBuilder;

use crate::app::session::Session;
use crate::app::widgets::TrackPreview;
use crate::app::widgets::TrackView;

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
                ui.label(self.session.name());

                let item_spacing_x = ui.style().spacing.item_spacing.x;
                ui.style_mut().spacing.item_spacing.x = 0.0;

                let tablebuilder = TableBuilder::new(ui)
                    .column(Column::auto_with_initial_suggestion(192.0))
                    .resizable(true)
                    .column(Column::remainder());

                tablebuilder.body(|body| {
                    body.rows(30.0, self.session.tracks().len(), |mut row| {
                        let index = row.index();
                        let track = &mut self.session.tracks_mut()[index];
                        row.col(|ui| {
                            ui.style_mut().spacing.item_spacing.x = item_spacing_x;
                            ui.add(TrackView::new(index, track));
                        });
                        row.col(|ui| {
                            ui.style_mut().spacing.item_spacing.x = item_spacing_x;
                            ui.add(TrackPreview::new(track));
                        });
                    });
                });
            })
            .response
    }
}
