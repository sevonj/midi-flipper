// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Color32;
use egui::Painter;
use egui::Pos2;
use egui::Rect;
use egui::Stroke;
use egui::Vec2;
use egui::Widget;

use crate::app::data::SessionTrack;

pub struct TrackPreview<'a> {
    index: usize,
    track: &'a mut SessionTrack,
    paint_position: Pos2,
    paint_scale: Vec2,
    clip_rect: Rect,
}

impl<'a> TrackPreview<'a> {
    pub fn new(
        index: usize,
        track: &'a mut SessionTrack,
        paint_position: Pos2,
        paint_scale: Vec2,
        clip_rect: Rect,
    ) -> Self {
        Self {
            index,
            track,
            paint_position,
            paint_scale,
            clip_rect,
        }
    }
}

impl Widget for TrackPreview<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let height = ui.available_height();
        let width = ui.available_width();
        ui.set_height(height);
        ui.set_width(width);

        let index = self.index;
        let color_note = track_color(index);
        let color_note_disabled = Color32::from_hex("#7777").unwrap();
        let stroke_note = Stroke::new(2.0, color_note);
        let stroke_note_disabled = Stroke::new(2.0, color_note_disabled);

        let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), self.clip_rect);
        if self.track.flip_enabled() {
            for note in self.track.track_original().paint_cache() {
                let a = self.paint_position + note.points()[0] * self.paint_scale;
                let b = self.paint_position + note.points()[1] * self.paint_scale;
                painter.line(vec![a, b], stroke_note_disabled);
            }

            for note in self.track.track_flipped().paint_cache() {
                let a = self.paint_position + note.points()[0] * self.paint_scale;
                let b = self.paint_position + note.points()[1] * self.paint_scale;
                painter.line(vec![a, b], stroke_note);
            }
        } else {
            for note in self.track.track_original().paint_cache() {
                let a = self.paint_position + note.points()[0] * self.paint_scale;
                let b = self.paint_position + note.points()[1] * self.paint_scale;
                painter.line(vec![a, b], stroke_note);
            }
        }

        ui.response()
    }
}

fn track_color(index: usize) -> Color32 {
    match index % 16 {
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
    .unwrap()
}
