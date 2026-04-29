// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Color32;
use egui::Painter;
use egui::Pos2;
use egui::Rect;
use egui::Stroke;
use egui::Vec2;
use egui::Widget;

use crate::app::data::PaintableNote;
use crate::app::data::SessionTrack;

pub struct TrackPreview<'a> {
    index: usize,
    track: &'a mut SessionTrack,
    paint_position: Pos2,
    paint_scale: Vec2,
    clip_rect: Rect,
    playing_original: bool,
}

impl<'a> TrackPreview<'a> {
    pub fn new(
        index: usize,
        track: &'a mut SessionTrack,
        paint_position: Pos2,
        paint_scale: Vec2,
        clip_rect: Rect,
        playing_original: bool,
    ) -> Self {
        Self {
            index,
            track,
            paint_position,
            paint_scale,
            clip_rect,
            playing_original,
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
        let stroke_width = (self.paint_scale.y / 128.0).max(2.0).round();
        let stroke_note = Stroke::new(stroke_width, color_note);
        let stroke_note_disabled = Stroke::new(stroke_width, color_note_disabled);

        let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), self.clip_rect);
        if self.track.flip_enabled() {
            let (stroke_og, stroke_flip) = if self.playing_original {
                (stroke_note, stroke_note_disabled)
            } else {
                (stroke_note_disabled, stroke_note)
            };
            for note in self.track.track_original().paint_cache() {
                painter.line(self.points(note), stroke_og);
            }
            for note in self.track.track_flipped().paint_cache() {
                painter.line(self.points(note), stroke_flip);
            }
        } else {
            for note in self.track.track_original().paint_cache() {
                painter.line(self.points(note), stroke_note);
            }
        }

        ui.response()
    }
}

impl TrackPreview<'_> {
    fn points(&self, note: &PaintableNote) -> Vec<Pos2> {
        let a = (self.paint_position + note.points()[0] * self.paint_scale).round();
        let mut b = (self.paint_position + note.points()[1] * self.paint_scale).round();
        b.x = b.x.max(a.x + 1.0);
        let points = vec![a, b];
        points
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
