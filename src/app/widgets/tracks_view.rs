// SPDX-License-Identifier: AGPL-3.0-or-later

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
use crate::app::widgets::StatusPage;
use crate::app::widgets::TrackPreview;
use crate::app::widgets::TrackView;

const DEFAULT_ZOOM: f32 = 8.0;
const TOP_HEIGHT: f32 = 24.0;
const MIN_TRACK_HEIGHT: f32 = 96.0;
const MAX_TRACK_HEIGHT: f32 = MIN_TRACK_HEIGHT * 4.0;

#[derive(Debug, Clone)]
struct ViewPortState {
    time_zoom: f32,
    time_off: f32,
    scroll_off: f32,
    track_height: f32,
}

impl Default for ViewPortState {
    fn default() -> Self {
        Self {
            time_zoom: DEFAULT_ZOOM,
            time_off: 0.0,
            scroll_off: 0.0,
            track_height: MIN_TRACK_HEIGHT,
        }
    }
}

impl ViewPortState {
    pub fn horizontal_zoom(
        &mut self,
        delta: f32,
        rect: Rect,
        relative_cursor_pos: Vec2,
        time_scale: f32,
    ) {
        let old_zoom = self.time_zoom;

        if delta > 0.0 {
            self.time_zoom *= 1.0 + delta * 0.2;
        } else {
            self.time_zoom /= 1.0 - delta * 0.2;
        }

        let old_len = rect.width() / old_zoom;
        let new_len = rect.width() / self.time_zoom;
        let delta_len = (new_len - old_len) * time_scale;
        self.time_off -= delta_len * relative_cursor_pos.x / rect.width();
    }

    pub fn vertical_zoom(
        &mut self,
        delta: f32,
        rect: Rect,
        num_tracks: usize,
        relative_cursor_pos: Vec2,
    ) {
        let old_height = TOP_HEIGHT + num_tracks as f32 * self.track_height;

        if delta > 0.0 {
            self.track_height *= 1.0 + delta * 0.1;
        } else {
            self.track_height /= 1.0 - delta * 0.1;
        }
        self.track_height = self.track_height.clamp(MIN_TRACK_HEIGHT, MAX_TRACK_HEIGHT);

        let new_height = TOP_HEIGHT + num_tracks as f32 * self.track_height;
        self.scroll_off *= new_height / old_height;
        self.scroll_off += (rect.height() * new_height / old_height - rect.height())
            * relative_cursor_pos.y
            / rect.height();
    }
}

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

        let state_id = ui.id().with("tracks_viewport_state");
        let mut state = ui.data_mut(|d| d.get_temp::<ViewPortState>(state_id).unwrap_or_default());
        let midi_time_scale = match &self.session.midi_header().division {
            midi_msg::Division::TicksPerQuarterNote(ticks) => *ticks as f32,
            midi_msg::Division::TimeCode {
                ticks_per_frame, ..
            } => *ticks_per_frame as f32,
        };

        let style = ui.global_style();
        let weak_bg_fill = style.visuals.widgets.open.weak_bg_fill;

        Panel::left("track_controls")
            .frame(Frame::NONE.fill(weak_bg_fill))
            .show_inside(ui, |ui| {
                let item_spacing = ui.style().spacing.item_spacing;
                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                Frame::group(&style)
                    .inner_margin(0.)
                    .outer_margin(0.)
                    .corner_radius(0.)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_height(TOP_HEIGHT);
                            ui.set_width(ui.available_width());
                            ui.style_mut().spacing.item_spacing = item_spacing;

                            if ui.button("Reset Zoom").clicked() {
                                state.time_zoom = DEFAULT_ZOOM;
                            }
                            if ui.button("Reset Position").clicked() {
                                state.time_off = 0.0;
                            }
                        });
                    });

                let scroll_resp = ScrollArea::vertical()
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .scroll_offset(vec2(0.0, state.scroll_off))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for (index, track) in self.session.tracks_mut().iter_mut().enumerate() {
                                ui.scope(|ui| {
                                    ui.set_height(state.track_height);
                                    ui.style_mut().spacing.item_spacing = item_spacing;
                                    ui.add(TrackView::new(index, track));
                                });
                            }
                        })
                    });
                state.scroll_off = scroll_resp.state.offset.y;
                scroll_resp.inner
            });

        if self.session.is_placeholder() {
            ui.data_mut(|d| d.insert_temp(state_id, ViewPortState::default()));
            return ui.add(StatusPage::status_nothing_open());
        }

        let response = CentralPanel::default()
            .frame(Frame::NONE)
            .show_inside(ui, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);
                ui.set_width(length * state.time_zoom);
                let faint_bg_color = ui.global_style().visuals.faint_bg_color;

                let col_major = Color32::from_hex("#7777").unwrap();
                let col_minor = Color32::from_hex("#7773").unwrap();
                let stroke_major = Stroke::new(1., col_major);
                let stroke_minor = Stroke::new(1., col_minor);

                let viewport_time_off = vec2(state.time_off, 0.0);
                let tracks_clip_rect = {
                    let rect = ui.available_rect_before_wrap();
                    rect.with_min_y(rect.min.y + TOP_HEIGHT)
                };
                let tracks_painter =
                    Painter::new(ui.ctx().clone(), ui.layer_id(), tracks_clip_rect);
                let view_time_scale = state.time_zoom / midi_time_scale;
                let tracks_scale = vec2(view_time_scale, state.track_height);
                let tracks_position = tracks_clip_rect.min - vec2(0.0, state.scroll_off);
                let tracks_area_size = tracks_clip_rect.size();

                let meter_clip_rect = {
                    let rect = ui.available_rect_before_wrap();
                    rect.with_max_y(rect.min.y + TOP_HEIGHT)
                };
                let meter_painter = Painter::new(ui.ctx().clone(), ui.layer_id(), meter_clip_rect);

                // Tracks stripe BG
                for i in 0..self.session.tracks().len() {
                    let min = tracks_position + vec2(0.0, i as f32 * tracks_scale.y);
                    let max = tracks_position
                        + vec2(ui.available_width(), (i + 1) as f32 * tracks_scale.y);

                    if i % 2 == 1 {
                        tracks_painter.rect(
                            Rect::from_min_max(min, max),
                            0.0,
                            faint_bg_color,
                            Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                    }
                }

                // Tracks Bar lines
                {
                    let bar_scale = vec2(view_time_scale, tracks_area_size.y);
                    for bar in self.session.beats_paint_cache() {
                        let a = tracks_clip_rect.min + (bar.0[0] - viewport_time_off) * bar_scale;
                        let b = tracks_clip_rect.min + (bar.0[1] - viewport_time_off) * bar_scale;
                        tracks_painter
                            .line(vec![a, b], if bar.1 { stroke_major } else { stroke_minor });
                    }
                }

                // Tracks
                for (index, track) in self.session.tracks_mut().iter_mut().enumerate() {
                    let track_row_offset = vec2(0.0, index as f32 * tracks_scale.y);
                    let position =
                        tracks_position - viewport_time_off * tracks_scale + track_row_offset;

                    ui.add(TrackPreview::new(
                        index,
                        track,
                        position,
                        tracks_scale,
                        tracks_clip_rect,
                    ));
                }

                // Meter Fill
                meter_painter.rect_filled(meter_clip_rect, 0.0, faint_bg_color);

                // Meter Events
                for (time, event) in self.session.marker_events() {
                    const ANCHOR: Align2 = Align2::LEFT_TOP;
                    let pos = meter_clip_rect.min - viewport_time_off * tracks_scale
                        + vec2(*time as f32, 0.0) * tracks_scale;
                    let font_id = FontId::monospace(10.0);
                    match event {
                        midi_msg::Meta::Marker(_) => (),
                        midi_msg::Meta::CuePoint(_) => (),
                        midi_msg::Meta::EndOfTrack => (),
                        midi_msg::Meta::SetTempo(tempo) => {
                            let bpm = 60_000_000 / tempo;
                            let text = format!("{bpm}bpm");
                            meter_painter.text(
                                pos + vec2(0.0, 12.0),
                                ANCHOR,
                                text,
                                font_id,
                                col_major,
                            );
                        }
                        midi_msg::Meta::TimeSignature(ts) => {
                            let text = format!("{}/{}", ts.numerator, ts.denominator);
                            meter_painter.text(pos, ANCHOR, text, font_id, col_major);
                        }
                        _ => continue,
                    }
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
                let num_tracks = self.session.tracks().len();
                let rect = response.rect;
                let relative_cursor_pos = ui.input(|ui| ui.pointer.hover_pos().unwrap()) - rect.min;

                if modifiers.is_none() {
                    // Scroll vertically
                    state.scroll_off -= delta * 50.0;
                } else if modifiers.alt {
                    // Scroll horizontally
                    state.time_off -= delta / state.time_zoom * midi_time_scale * 50.;
                } else {
                    // Zoom
                    if modifiers.shift {
                        state.vertical_zoom(delta, rect, num_tracks, relative_cursor_pos);
                    }
                    if modifiers.ctrl {
                        state.horizontal_zoom(delta, rect, relative_cursor_pos, midi_time_scale);
                    }
                }

                state.time_off = state.time_off.clamp(0.0, length);
                let scroll_max = (TOP_HEIGHT + num_tracks as f32 * state.track_height
                    - response.rect.height())
                .max(0.0);

                state.scroll_off = state.scroll_off.clamp(0.0, scroll_max);
            }
        }

        ui.data_mut(|d| d.insert_temp(state_id, state));

        response
    }
}
