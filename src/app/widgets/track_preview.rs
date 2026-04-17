use egui::Color32;
use egui::Painter;
use egui::Sense;
use egui::Stroke;
use egui::Widget;
use egui::vec2;

use crate::app::data::SessionTrack;
use crate::app::data::TrackMidiData;

pub struct TrackPreview<'a> {
    index: usize,
    track: &'a mut SessionTrack,
    zoom: &'a mut f32,
    offset: &'a mut f32,
}

impl<'a> TrackPreview<'a> {
    pub fn new(
        index: usize,
        track: &'a mut SessionTrack,
        zoom: &'a mut f32,
        offset: &'a mut f32,
    ) -> Self {
        Self {
            index,
            track,
            zoom,
            offset,
        }
    }
}

impl Widget for TrackPreview<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let height = ui.available_height();
        let width = ui.available_width();
        ui.set_height(height);
        ui.set_width(width);

        let color_og = Color32::from_hex("#7c6f64").unwrap();
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

        let pos = ui.next_widget_position();

        if self.track.flip_enabled() {
            self.paint_track(height, color_og, &painter, pos, self.track.track_original());
            self.paint_track(height, color, &painter, pos, self.track.track_flipped());
        } else {
            self.paint_track(height, color, &painter, pos, self.track.track_original());
        }

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        if response.hovered() {
            let mouse_wheel = ui.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::MouseWheel {
                        delta, modifiers, ..
                    } => Some((delta.y, *modifiers)),
                    _ => None,
                })
            });
            if let Some((delta, modifiers)) = mouse_wheel {
                if modifiers.ctrl {
                    let old_zoom = *self.zoom;
                    let mut new_zoom = old_zoom;
                    if delta > 0.0 {
                        new_zoom *= 1.0 + delta * 0.2;
                    } else {
                        new_zoom /= 1.0 - delta * 0.2;
                    }

                    *self.zoom = new_zoom;

                    let old_len = rect.width() / old_zoom;
                    let new_len = rect.width() / new_zoom;
                    let delta_len = new_len - old_len;

                    let relative_cursor_pos =
                        ui.input(|ui| ui.pointer.hover_pos().unwrap()) - rect.min;
                    *self.offset -= delta_len * relative_cursor_pos.x / rect.width();
                } else if modifiers.alt {
                    *self.offset -= delta / *self.zoom * 50.;
                }
            }
        }

        response
    }
}

impl TrackPreview<'_> {
    fn paint_track(
        &self,
        height: f32,
        color: Color32,
        painter: &Painter,
        pos: egui::Pos2,
        track: &TrackMidiData,
    ) {
        let stroke = Stroke::new(2.0, color);
        for note in track.paint_cache() {
            let offset = vec2(*self.offset, 0.0);
            let mult = vec2(*self.zoom, height / 128.0);

            let a = pos + (note.points()[0] - offset) * mult;
            let b = pos + (note.points()[1] - offset) * mult;

            painter.line(vec![a, b], stroke);
        }
    }
}
