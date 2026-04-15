use egui::{Frame, Stroke, Widget};

use crate::app::session::SessionTrack;

const HEIGHT: f32 = 64.0;

pub struct TrackPreview<'a> {
    track: &'a mut SessionTrack,
}

impl<'a> TrackPreview<'a> {
    pub fn new(track: &'a mut SessionTrack) -> Self {
        Self { track }
    }
}

impl Widget for TrackPreview<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let style = ui.style();
            let bg_fill = style.visuals.widgets.open.bg_fill;

            Frame::group(&style)
                .inner_margin(0.)
                .outer_margin(0.)
                .corner_radius(0.)
                .stroke(Stroke::NONE)
                .fill(bg_fill)
                .show(ui, |ui| {
                    ui.set_height(HEIGHT);
                    ui.set_width(ui.available_width());
                })
        })
        .response
    }
}
