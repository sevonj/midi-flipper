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
use egui::Sense;
use egui::Stroke;
use egui::UiBuilder;
use egui::Vec2;
use egui::Widget;
use egui::scroll_area::ScrollBarVisibility;
use egui::vec2;

use crate::app::data::Session;
use crate::app::shortcuts::SHORTCUT_VP_START;
use crate::app::shortcuts::SHORTCUT_VP_START_ALT;
use crate::app::shortcuts::SHORTCUT_VP_ZOOM_H_IN;
use crate::app::shortcuts::SHORTCUT_VP_ZOOM_H_OUT;
use crate::app::shortcuts::SHORTCUT_VP_ZOOM_RESET;
use crate::app::shortcuts::SHORTCUT_VP_ZOOM_V_IN;
use crate::app::shortcuts::SHORTCUT_VP_ZOOM_V_OUT;
use crate::app::widgets::StatusPage;
use crate::app::widgets::TrackPreview;
use crate::app::widgets::TrackView;

const DEFAULT_ZOOM: f32 = 8.0;
const TOP_HEIGHT: f32 = 24.0;
const MIN_TRACK_HEIGHT: f32 = 76.0;
const MAX_TRACK_HEIGHT: f32 = 128.0 * 8.0;

#[derive(Debug, Clone)]
struct TimelineState {
    time_zoom: f32,
    time_off: f32,
    scroll_off: f32,
    track_height: f32,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            time_zoom: DEFAULT_ZOOM,
            time_off: 0.0,
            scroll_off: 0.0,
            track_height: MIN_TRACK_HEIGHT,
        }
    }
}

impl TimelineState {
    pub fn reset_zoom(&mut self) {
        self.time_zoom = DEFAULT_ZOOM;
        self.track_height = MIN_TRACK_HEIGHT;
    }

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
        relative_cursor_pos: Vec2,
        num_tracks: usize,
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

pub struct Timeline<'a> {
    session: &'a mut Session,
}

impl<'a> Timeline<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session }
    }
}

impl Timeline<'_> {
    pub fn clear_state(ui: &mut egui::Ui) {
        ui.data_mut(|d| d.remove_temp::<TimelineState>(Self::state_id(ui)));
    }

    fn state_id(ui: &egui::Ui) -> egui::Id {
        ui.id().with("tracks_timeline_state")
    }
}

impl Widget for Timeline<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let length = self.session.length();

        let state_id = Self::state_id(ui);
        let mut state = ui.data_mut(|d| d.get_temp::<TimelineState>(state_id).unwrap_or_default());
        let midi_time_scale = match &self.session.midi_header().division {
            midi_msg::Division::TicksPerQuarterNote(ticks) => *ticks as f32,
            midi_msg::Division::TimeCode {
                ticks_per_frame, ..
            } => *ticks_per_frame as f32,
        };
        let view_time_scale = state.time_zoom / midi_time_scale;

        let style = ui.global_style();
        let weak_bg_fill = style.visuals.widgets.open.weak_bg_fill;

        let mut timeline_rect = Rect::ZERO;
        let response = CentralPanel::default()
            .frame(Frame::NONE)
            .show_inside(ui, |ui| {
                Panel::left("track_controls")
                    .min_size(220.0)
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
                                    ui.add_space(2.0);

                                    ui.menu_button("ℹ", |ui| {
                                        ui.strong("Viewport Controls");
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.weak("Scroll");
                                                ui.weak("Alt+Scroll");
                                                ui.weak("Ctrl+Scroll");
                                                ui.weak("Shift+Scroll");
                                            });
                                            ui.vertical(|ui| {
                                                ui.label("Scroll vertically");
                                                ui.label("Scroll horizontally");
                                                ui.label("Zoom horizontally");
                                                ui.label("Zoom vertically");
                                            });
                                        });

                                        ui.separator();

                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.weak(ui.format_shortcut(&SHORTCUT_VP_ZOOM_H_IN));
                                                ui.weak(
                                                    ui.format_shortcut(&SHORTCUT_VP_ZOOM_H_OUT),
                                                );
                                                ui.weak(ui.format_shortcut(&SHORTCUT_VP_ZOOM_V_IN));
                                                ui.weak(
                                                    ui.format_shortcut(&SHORTCUT_VP_ZOOM_V_OUT),
                                                );
                                            });
                                            ui.vertical(|ui| {
                                                ui.label("Zoom in horizontally");
                                                ui.label("Zoom out horizontally");
                                                ui.label("Zoom in vertically");
                                                ui.label("Zoom out vertically");
                                            });
                                        });

                                        ui.separator();

                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.weak(
                                                    ui.format_shortcut(&SHORTCUT_VP_ZOOM_RESET),
                                                );
                                            });
                                            ui.vertical(|ui| {
                                                ui.label("Reset zoom");
                                            });
                                        });

                                        ui.separator();

                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.weak(format!(
                                                    "{} or {}",
                                                    ui.format_shortcut(&SHORTCUT_VP_START),
                                                    ui.format_shortcut(&SHORTCUT_VP_START_ALT)
                                                ));
                                            });
                                            ui.vertical(|ui| {
                                                ui.label("Go to start");
                                            });
                                        });
                                    });
                                });
                            });

                        ScrollArea::vertical()
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                            .wheel_scroll_multiplier(Vec2::splat(0.0))
                            .scroll_offset(vec2(0.0, state.scroll_off))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    for (index, track) in
                                        self.session.tracks_mut().iter_mut().enumerate()
                                    {
                                        ui.scope(|ui| {
                                            ui.set_height(state.track_height);
                                            ui.style_mut().spacing.item_spacing = item_spacing;
                                            ui.add(TrackView::new(index, track));
                                        });
                                    }
                                })
                            })
                            .inner
                    });

                if self.session.is_placeholder() {
                    ui.data_mut(|d| d.insert_temp(state_id, TimelineState::default()));
                    return ui.add(StatusPage::status_nothing_open());
                }

                let timeline_resp = ui
                    .scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
                        CentralPanel::default()
                            .frame(Frame::NONE)
                            .show_inside(ui, |ui| {
                                timeline_rect = ui.available_rect_before_wrap();

                                ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);
                                ui.set_width(length * state.time_zoom);

                                let faint_bg_color = ui.global_style().visuals.faint_bg_color;
                                let col_major = Color32::from_hex("#7777").unwrap();
                                let col_minor = Color32::from_hex("#7773").unwrap();
                                let col_cursor = Color32::from_hex("#55cc5577").unwrap();
                                let col_playhead = Color32::from_hex("#55cc55").unwrap();
                                let stroke_major = Stroke::new(1., col_major);
                                let stroke_minor = Stroke::new(1., col_minor);
                                let stroke_cursor = Stroke::new(1., col_cursor);
                                let stroke_cursor_top = Stroke::new(4., col_cursor);
                                let stroke_playhead = Stroke::new(1., col_playhead);
                                let stroke_playhead_top = Stroke::new(4., col_playhead);

                                let timeline_start = vec2(state.time_off, 0.0);
                                let tracks_clip_rect =
                                    timeline_rect.with_min_y(timeline_rect.min.y + TOP_HEIGHT);
                                let tracks_painter =
                                    Painter::new(ui.ctx().clone(), ui.layer_id(), tracks_clip_rect);
                                let tracks_scale = vec2(view_time_scale, state.track_height);
                                let tracks_position =
                                    tracks_clip_rect.min - vec2(0.0, state.scroll_off);
                                let tracks_area_size = tracks_clip_rect.size();

                                let meter_clip_rect =
                                    timeline_rect.with_max_y(timeline_rect.min.y + TOP_HEIGHT);

                                let meter_painter =
                                    Painter::new(ui.ctx().clone(), ui.layer_id(), meter_clip_rect);

                                // Tracks stripe BG
                                for i in 0..self.session.tracks().len() {
                                    let min =
                                        tracks_position + vec2(0.0, i as f32 * tracks_scale.y);
                                    let max = tracks_position
                                        + vec2(
                                            ui.available_width(),
                                            (i + 1) as f32 * tracks_scale.y,
                                        );

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
                                        let a = tracks_clip_rect.min
                                            + (bar.0[0] - timeline_start) * bar_scale;
                                        let b = tracks_clip_rect.min
                                            + (bar.0[1] - timeline_start) * bar_scale;
                                        tracks_painter.line(
                                            vec![
                                                a.round() - Vec2::splat(0.5),
                                                b.round() - Vec2::splat(0.5),
                                            ],
                                            if bar.1 { stroke_major } else { stroke_minor },
                                        );
                                    }
                                }

                                // Tracks
                                let playing_original = self.session.playback_original();
                                for (index, track) in
                                    self.session.tracks_mut().iter_mut().enumerate()
                                {
                                    let track_row_offset = vec2(0.0, index as f32 * tracks_scale.y);
                                    let position = tracks_position - timeline_start * tracks_scale
                                        + track_row_offset;

                                    ui.add(TrackPreview::new(
                                        index,
                                        track,
                                        position,
                                        tracks_scale,
                                        tracks_clip_rect,
                                        playing_original,
                                    ));
                                }

                                // Meter Fill
                                meter_painter.rect_filled(meter_clip_rect, 0.0, faint_bg_color);

                                // Meter Events
                                for (time, event) in self.session.marker_events() {
                                    const ANCHOR: Align2 = Align2::LEFT_TOP;
                                    let pos = meter_clip_rect.min - timeline_start * tracks_scale
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
                                            let text =
                                                format!("{}/{}", ts.numerator, ts.denominator);
                                            meter_painter
                                                .text(pos, ANCHOR, text, font_id, col_major);
                                        }
                                        _ => continue,
                                    }
                                }

                                // Cursor and Playhead
                                {
                                    let line_scale = vec2(view_time_scale, tracks_area_size.y);

                                    let cursor_pos = self.session.cursor_pos().max(1.0);
                                    let a = tracks_clip_rect.min
                                        + (vec2(cursor_pos, 0.0) - timeline_start) * line_scale;
                                    let b = tracks_clip_rect.min
                                        + (vec2(cursor_pos, 1.0) - timeline_start) * line_scale;
                                    tracks_painter.line(
                                        vec![
                                            a.round() - Vec2::splat(0.5),
                                            b.round() - Vec2::splat(0.5),
                                        ],
                                        stroke_cursor,
                                    );

                                    let a = meter_clip_rect.min
                                        + (vec2(cursor_pos, 0.0) - timeline_start) * line_scale
                                        + vec2(-4.0, TOP_HEIGHT);
                                    let b = meter_clip_rect.min
                                        + (vec2(cursor_pos, 0.0) - timeline_start) * line_scale
                                        + vec2(4.0, TOP_HEIGHT);

                                    meter_painter.line(
                                        vec![
                                            a.round() - Vec2::splat(0.5),
                                            b.round() - Vec2::splat(0.5),
                                        ],
                                        stroke_cursor_top,
                                    );

                                    if self.session.is_playback_in_progress() {
                                        let time = self.session.playback_position().as_secs_f32();

                                        for bar in self.session.cached_bars() {
                                            if bar.end_time < time {
                                                continue;
                                            }
                                            let playhead_pos = bar.start_position.max(1.0);

                                            let a = tracks_clip_rect.min
                                                + (vec2(playhead_pos, 0.0) - timeline_start)
                                                    * line_scale;
                                            let b = tracks_clip_rect.min
                                                + (vec2(playhead_pos, 1.0) - timeline_start)
                                                    * line_scale;
                                            tracks_painter.line(
                                                vec![
                                                    a.round() - Vec2::splat(0.5),
                                                    b.round() - Vec2::splat(0.5),
                                                ],
                                                stroke_playhead,
                                            );

                                            let a = meter_clip_rect.min
                                                + (vec2(playhead_pos, 0.0) - timeline_start)
                                                    * line_scale
                                                + vec2(-4.0, TOP_HEIGHT);
                                            let b = meter_clip_rect.min
                                                + (vec2(playhead_pos, 0.0) - timeline_start)
                                                    * line_scale
                                                + vec2(4.0, TOP_HEIGHT);

                                            meter_painter.line(
                                                vec![
                                                    a.round() - Vec2::splat(0.5),
                                                    b.round() - Vec2::splat(0.5),
                                                ],
                                                stroke_playhead_top,
                                            );

                                            break;
                                        }
                                    }
                                }
                            })
                            .response
                    })
                    .response;

                if timeline_resp.clicked() {
                    let rect = timeline_rect;
                    let relative_cursor_pos =
                        ui.input(|ui| ui.pointer.hover_pos().unwrap_or_default()) - rect.min;
                    let position =
                        state.time_off + relative_cursor_pos.x * midi_time_scale / state.time_zoom;
                    self.session.set_cursor_pos(position);
                }

                timeline_resp
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
            let rect = timeline_rect;
            let relative_cursor_pos =
                ui.input(|ui| ui.pointer.hover_pos().unwrap_or_default()) - rect.min;
            let num_tracks = self.session.tracks().len();

            if let Some((delta, modifiers)) = mouse_wheel {
                if modifiers.is_none() {
                    // Scroll vertically
                    state.scroll_off -= delta * 50.0;
                } else if modifiers.alt {
                    // Scroll horizontally
                    state.time_off -= delta / state.time_zoom * midi_time_scale * 50.;
                } else {
                    // Zoom
                    if modifiers.ctrl {
                        state.horizontal_zoom(delta, rect, relative_cursor_pos, midi_time_scale);
                    }
                    if modifiers.shift {
                        state.vertical_zoom(delta, rect, relative_cursor_pos, num_tracks);
                    }
                }

                state.time_off = state.time_off.clamp(0.0, length);
                let scroll_max =
                    (TOP_HEIGHT + num_tracks as f32 * state.track_height - rect.height()).max(0.0);

                state.scroll_off = state.scroll_off.clamp(0.0, scroll_max);
            } else if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_VP_ZOOM_RESET)) {
                state.reset_zoom();
            } else if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_VP_ZOOM_H_IN)) {
                state.horizontal_zoom(4.0, rect, rect.size() / 2.0, midi_time_scale);
            } else if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_VP_ZOOM_H_OUT)) {
                state.horizontal_zoom(-4.0, rect, rect.size() / 2.0, midi_time_scale);
            } else if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_VP_ZOOM_V_IN)) {
                state.vertical_zoom(4.0, rect, rect.size() / 2.0, num_tracks);
            } else if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_VP_ZOOM_V_OUT)) {
                state.vertical_zoom(-4.0, rect, rect.size() / 2.0, num_tracks);
            } else if ui.input_mut(|i| {
                i.consume_shortcut(&SHORTCUT_VP_START) || i.consume_shortcut(&SHORTCUT_VP_START_ALT)
            }) {
                state.time_off = 0.0;
            }
        }
        ui.data_mut(|d| d.insert_temp(state_id, state));

        response
    }
}
