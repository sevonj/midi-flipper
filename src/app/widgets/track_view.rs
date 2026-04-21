// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Button;
use egui::Frame;
use egui::Label;
use egui::RichText;
use egui::Vec2;
use egui::Widget;

use crate::app::data::SessionTrack;

const BUTTON_MIN_SIZE: Vec2 = Vec2::splat(20.0);

pub struct TrackView<'a> {
    index: usize,
    track: &'a mut SessionTrack,
}

impl<'a> TrackView<'a> {
    pub fn new(index: usize, track: &'a mut SessionTrack) -> Self {
        Self { index, track }
    }
}

impl Widget for TrackView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let style = ui.style();
        let weak_bg_fill = style.visuals.widgets.open.weak_bg_fill;

        Frame::group(style)
            .inner_margin(0.)
            .outer_margin(0.)
            .corner_radius(0.)
            .fill(weak_bg_fill)
            .show(ui, |ui| {
                let height = ui.available_height();

                ui.horizontal(|ui| {
                    ui.set_height(height);
                    ui.set_width(ui.available_width());

                    ui.horizontal_centered(|ui| {
                        ui.set_width(18.);
                        ui.add_space(4.);
                        ui.label(self.index.to_string());
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(1.0);
                        ui.add_space(2.0);

                        ui.add(
                            Label::new(track_name(self.track))
                                .wrap_mode(egui::TextWrapMode::Truncate),
                        );

                        ui.horizontal(|ui| {
                            let flip_enabled = self.track.flip_enabled();
                            if toggle_button(ui, flip_enabled, "F", "Flip Track").clicked() {
                                self.track.set_flip_enabled(!flip_enabled);
                            }
                            let ignore_ch10 = self.track.ignore_ch10();
                            if toggle_button(ui, !ignore_ch10, "Ch10", "Flip Channel 10  (drums)")
                                .clicked()
                            {
                                self.track.set_ignore_ch10(!ignore_ch10);
                            }
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            let mut transposition = self.track.transposition();

                            ui.horizontal(|ui| {
                                ui.set_width(54.0);

                                if button(ui, "R", "Reset Transposition").clicked() {
                                    transposition = 0;
                                    self.track.set_transposition(transposition);
                                }

                                let sign = if transposition > 0 { "+" } else { "" };
                                let octaves = transposition / 12;
                                let oct_string = if octaves != 0 {
                                    format!("{octaves}:")
                                } else {
                                    String::new()
                                };
                                let semitones = transposition.abs() % 12;
                                ui.label(format!("{sign}{oct_string}{semitones}"))
                                    .on_hover_text("Transposition");
                            });

                            ui.horizontal(|ui| {
                                if button(ui, "+1", "Semitone Up").clicked() {
                                    transposition += 1;
                                    self.track.set_transposition(transposition);
                                }
                                if button(ui, "-1", "Semitone Down").clicked() {
                                    transposition -= 1;
                                    self.track.set_transposition(transposition);
                                }
                                if button(ui, "+Oct", "Octave Up").clicked() {
                                    transposition += 12;
                                    self.track.set_transposition(transposition);
                                }
                                if button(ui, "-Oct", "Octave Down").clicked() {
                                    transposition -= 12;
                                    self.track.set_transposition(transposition);
                                }
                            });

                            ui.add_space(4.0);
                        });
                    });
                })
            })
            .response
    }
}

fn toggle_button(ui: &mut egui::Ui, selected: bool, label: &str, tooltip: &str) -> egui::Response {
    ui.add(Button::selectable(selected, label).min_size(BUTTON_MIN_SIZE))
        .on_hover_text(tooltip)
}

fn button(ui: &mut egui::Ui, label: &str, tooltip: &str) -> egui::Response {
    ui.add(Button::new(label).min_size(BUTTON_MIN_SIZE))
        .on_hover_text(tooltip)
}

fn track_name(track: &mut SessionTrack) -> RichText {
    if !track.is_midi() {
        RichText::new("[unknown track type]").weak()
    } else if let Some(name) = track.name() {
        if name.is_empty() {
            RichText::new("[track name is empty]").weak()
        } else {
            RichText::new(name)
        }
    } else {
        RichText::new("[unnamed track]").weak()
    }
}
