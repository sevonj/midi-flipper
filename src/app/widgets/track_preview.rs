use egui::Color32;
use egui::Painter;
use egui::Stroke;
use egui::Widget;
use egui::vec2;

use crate::app::data::SessionTrack;

pub struct TrackPreview<'a> {
    index: usize,
    track: &'a mut SessionTrack,
}

impl<'a> TrackPreview<'a> {
    pub fn new(index: usize, track: &'a mut SessionTrack) -> Self {
        Self { index, track }
    }
}

impl Widget for TrackPreview<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let height = ui.available_height();
            ui.set_height(height);
            ui.set_width(ui.available_width());

            let color = match self.index % 16 {
                0 => Color32::from_hex("#BC3B2B"),
                1 => Color32::from_hex("#979634"),
                2 => Color32::from_hex("#CE9B3B"),
                3 => Color32::from_hex("#548286"),
                4 => Color32::from_hex("#A66684"),
                5 => Color32::from_hex("#729A6D"),
                6 => Color32::from_hex("#A49885"),
                7 => Color32::from_hex("#C86529"),
                8 => Color32::from_hex("#EA5A40"),
                9 => Color32::from_hex("#B8BA41"),
                10 => Color32::from_hex("#F2BE4A"),
                11 => Color32::from_hex("#88A297"),
                12 => Color32::from_hex("#C88999"),
                13 => Color32::from_hex("#96BD80"),
                14 => Color32::from_hex("#E8DBB4"),
                15 => Color32::from_hex("#F08736"),
                _ => unreachable!(),
            }
            .unwrap();

            let painter = Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                ui.available_rect_before_wrap(),
            );

            let stroke = Stroke::new(3.0, color);
            let pos = ui.next_widget_position();
            for note in self.track.note_paint_cache() {
                const TIME_MULT: f32 = 0.1;

                let mult = vec2(TIME_MULT, height / 128.0);

                let a = pos + (note.points()[0] * mult);
                let b = pos + (note.points()[1] * mult);

                painter.line(vec![a, b], stroke);
            }
        })
        .response
    }
}
