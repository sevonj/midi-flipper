use egui::CentralPanel;
use egui::Vec2;
use egui::Widget;
use egui_extras::Column;
use egui_extras::TableBuilder;

use crate::app::data::Session;
use crate::app::widgets::TrackPreview;
use crate::app::widgets::TrackView;

const HEIGHT: f32 = 64.0;

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

                let item_spacing = ui.style().spacing.item_spacing;
                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                let tablebuilder = TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto_with_initial_suggestion(192.0))
                    .resizable(true)
                    .column(Column::remainder());

                tablebuilder.body(|body| {
                    body.rows(HEIGHT, self.session.tracks().len(), |mut row| {
                        let index = row.index();
                        let track = &mut self.session.tracks_mut()[index];
                        row.col(|ui| {
                            ui.style_mut().spacing.item_spacing = item_spacing;
                            ui.add(TrackView::new(index, track));
                        });
                        row.col(|ui| {
                            ui.style_mut().spacing.item_spacing = item_spacing;
                            ui.add(TrackPreview::new(index, track));
                        });
                    });
                });
            })
            .response
    }
}
