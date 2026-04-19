use egui::Align2;
use egui::CentralPanel;
use egui::Color32;
use egui::FontId;
use egui::Frame;
use egui::Painter;
use egui::Panel;
use egui::Rect;
use egui::ScrollArea;
use egui::Stroke;
use egui::Vec2;
use egui::Widget;
use egui::scroll_area::ScrollBarVisibility;
use egui::vec2;

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
            d.get_temp::<(f32, Vec2)>(state_id)
                .unwrap_or((DEFAULT_ZOOM, Vec2::splat(0.0)))
        });

        let scroll_offset_id = ui.id().with("scroll_offset");
        let mut scroll_offset = ui.data_mut(|d| {
            d.get_temp::<Vec2>(scroll_offset_id)
                .unwrap_or(Vec2::splat(0.0))
        });

        Panel::top("tracks_top")
            .frame(Frame::NONE)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Reset Zoom").clicked() {
                        zoom = DEFAULT_ZOOM;
                    }
                    if ui.button("Reset Position").clicked() {
                        offset.x = 0.0;
                    }
                });
            });

        Panel::left("track_controls")
            .frame(Frame::NONE)
            .show_inside(ui, |ui| {
                let scroll_resp = ScrollArea::vertical()
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .scroll_offset(scroll_offset)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            let item_spacing = ui.style().spacing.item_spacing;
                            ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                            for (index, track) in self.session.tracks_mut().iter_mut().enumerate() {
                                ui.scope(|ui| {
                                    ui.set_height(HEIGHT);
                                    ui.style_mut().spacing.item_spacing = item_spacing;
                                    ui.add(TrackView::new(index, track));
                                });
                            }
                        })
                    });
                scroll_offset = scroll_resp.state.offset;
                scroll_resp.inner
            });

        let response = CentralPanel::default()
            .frame(Frame::NONE)
            .show_inside(ui, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);
                ui.set_width(length * zoom);
                let faint_bg_color = ui.global_style().visuals.faint_bg_color;

                let col_major = Color32::from_hex("#7777").unwrap();
                let col_minor = Color32::from_hex("#7773").unwrap();
                let stroke_major = Stroke::new(2., col_major);
                let stroke_minor = Stroke::new(1., col_minor);

                let clip_rect = ui.available_rect_before_wrap();
                let mult = vec2(zoom, ui.available_height());

                let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), clip_rect);

                let scale = vec2(zoom, HEIGHT);
                let position = clip_rect.min - scroll_offset;

                // Striped track bg
                for i in 0..self.session.tracks().len() {
                    let min = position + vec2(0.0, i as f32 * scale.y);
                    let max = position + vec2(ui.available_width(), (i + 1) as f32 * scale.y);

                    if i % 2 == 1 {
                        painter.rect(
                            Rect::from_min_max(min, max),
                            0.0,
                            faint_bg_color,
                            Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                    }
                }

                // Bar lines
                for bar in self.session.beats_paint_cache() {
                    let a = clip_rect.min + (bar.0[0] - offset) * mult;
                    let b = clip_rect.min + (bar.0[1] - offset) * mult;
                    painter.line(vec![a, b], if bar.1 { stroke_major } else { stroke_minor });
                }

                // Events
                for (time, event) in self.session.marker_events() {
                    const ANCHOR: Align2 = Align2::LEFT_TOP;
                    let pos = position - offset * scale + vec2(*time as f32, 0.0) * scale;
                    let font_id = FontId::monospace(8.0);
                    match event {
                        midi_msg::Meta::Marker(_) => (),
                        midi_msg::Meta::CuePoint(_) => (),
                        midi_msg::Meta::EndOfTrack => (),
                        midi_msg::Meta::SetTempo(tempo) => {
                            let bpm = 60_000_000 / tempo;
                            let text = format!("{bpm}bpm");
                            painter.text(pos + vec2(0.0, 8.0), ANCHOR, text, font_id, col_major);
                        }
                        midi_msg::Meta::TimeSignature(ts) => {
                            let text = format!("{}/{}", ts.numerator, ts.denominator);
                            painter.text(pos, ANCHOR, text, font_id, col_major);
                        }
                        _ => continue,
                    }
                }

                // Tracks
                for (index, track) in self.session.tracks_mut().iter_mut().enumerate() {
                    let track_row_offset = vec2(0.0, index as f32 * scale.y);
                    let position = position - offset * scale + track_row_offset;

                    ui.add(TrackPreview::new(index, track, position, scale, clip_rect));
                }
            })
            .response;

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
                if modifiers.is_none() {
                    scroll_offset.y -= delta * 50.0;
                } else if modifiers.ctrl {
                    let old_zoom = zoom;
                    let mut new_zoom = old_zoom;
                    if delta > 0.0 {
                        new_zoom *= 1.0 + delta * 0.2;
                    } else {
                        new_zoom /= 1.0 - delta * 0.2;
                    }

                    zoom = new_zoom;

                    let old_len = response.rect.width() / old_zoom;
                    let new_len = response.rect.width() / new_zoom;
                    let delta_len = new_len - old_len;

                    let relative_cursor_pos =
                        ui.input(|ui| ui.pointer.hover_pos().unwrap()) - response.rect.min;
                    offset.x -= delta_len * relative_cursor_pos.x / response.rect.width();
                } else if modifiers.alt {
                    offset.x -= delta / zoom * 50.;
                }
                offset.x = offset.x.clamp(0.0, length);
                let num_tracks = self.session.tracks().len();
                let scroll_max = (num_tracks as f32 * HEIGHT - response.rect.height()).max(0.0);
                scroll_offset.y = scroll_offset.y.clamp(0.0, scroll_max);
            }
        }

        ui.data_mut(|d| d.insert_temp(state_id, (zoom, offset)));
        ui.data_mut(|d| d.insert_temp(scroll_offset_id, scroll_offset));

        response
    }
}
