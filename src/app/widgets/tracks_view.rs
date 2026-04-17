// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::CentralPanel;
use egui::Color32;
use egui::Painter;
use egui::Stroke;
use egui::Vec2;
use egui::Widget;
use egui::vec2;
use egui_extras::Column;
use egui_extras::TableBuilder;

use crate::app::data::Session;
use crate::app::widgets::TrackPreview;
use crate::app::widgets::TrackView;

const HEIGHT: f32 = 96.0;

pub struct TracksView<'a> {
    session: &'a mut Session,
}

impl<'a> TracksView<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session }
    }
}

impl Widget for TracksView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let length = self.session.length();

        let state_id = ui.id().with("zoom");
        const DEFAULT_ZOOM: f32 = 1.0;
        let (mut zoom, mut offset) = ui.data_mut(|d| {
            d.get_temp::<(f32, f32)>(state_id)
                .unwrap_or((DEFAULT_ZOOM, 0.0))
        });

        ui.horizontal(|ui| {
            if ui.button("Reset Zoom").clicked() {
                zoom = DEFAULT_ZOOM;
            }
            if ui.button("Reset Position").clicked() {
                offset = 0.0;
            }
        });

        let response = CentralPanel::default()
            .show_inside(ui, |ui| {
                let item_spacing = ui.style().spacing.item_spacing;
                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                let tablebuilder = TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto_with_initial_suggestion(192.0))
                    .resizable(true)
                    .column(Column::remainder());

                let beats_paint_cache = self.session.beats_paint_cache().to_vec();

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
                            ui.set_width(length * zoom);

                            let color = Color32::from_hex("#7c6f6477").unwrap();
                            let color2 = Color32::from_hex("#7c6f643f").unwrap();
                            let painter = Painter::new(
                                ui.ctx().clone(),
                                ui.layer_id(),
                                ui.available_rect_before_wrap(),
                            );
                            let pos = ui.next_widget_position();
                            let offset2 = vec2(offset, 0.0);
                            let mult = vec2(zoom, HEIGHT);
                            let stroke = Stroke::new(2., color);
                            let stroke2 = Stroke::new(1., color2);

                            for line in &beats_paint_cache {
                                let a = pos + (line.0[0] - offset2) * mult;
                                let b = pos + (line.0[1] - offset2) * mult;

                                painter.line(vec![a, b], if line.1 { stroke } else { stroke2 });
                            }

                            ui.add(TrackPreview::new(index, track, &mut zoom, &mut offset));
                            offset = offset.clamp(0.0, length);
                        });
                    });
                });
            })
            .response;

        ui.data_mut(|d| d.insert_temp(state_id, (zoom, offset)));

        response
    }
}
