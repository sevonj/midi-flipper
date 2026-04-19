// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Checkbox;
use egui::Frame;
use egui::Label;
use egui::RichText;
use egui::Widget;

use crate::app::data::SessionTrack;

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
                        ui.style_mut().spacing.item_spacing.y = 1.0;

                        ui.horizontal(|ui| {
                            let mut flip_enabled = self.track.flip_enabled();
                            if ui.add(Checkbox::new(&mut flip_enabled, "")).changed() {
                                self.track.set_flip_enabled(flip_enabled);
                            }
                            ui.add(
                                Label::new(track_name(self.track))
                                    .wrap_mode(egui::TextWrapMode::Truncate),
                            )
                            .on_hover_text("Flip me?");
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            let mut transposition = self.track.transposition();

                            ui.horizontal(|ui| {
                                let sign = if transposition > 0 { "+" } else { "" };
                                let octaves = transposition / 12;
                                let oct_string = if octaves != 0 {
                                    format!("{octaves}:")
                                } else {
                                    String::new()
                                };
                                let semitones = transposition % 12;
                                ui.label(format!("Transposition: {sign}{oct_string}{semitones}"));
                            });

                            ui.horizontal(|ui| {
                                ui.style_mut().spacing.item_spacing.x = 1.0;

                                if ui.button("Reset").clicked() {
                                    transposition = 0;
                                    self.track.set_transposition(transposition);
                                }

                                ui.add_space(3.0);

                                if ui.button("+1").clicked() {
                                    transposition += 1;
                                    self.track.set_transposition(transposition);
                                }
                                if ui.button("-1").clicked() {
                                    transposition -= 1;
                                    self.track.set_transposition(transposition);
                                }

                                ui.add_space(3.0);

                                if ui.button("+Oct").clicked() {
                                    transposition += 12;
                                    self.track.set_transposition(transposition);
                                }
                                if ui.button("-Oct").clicked() {
                                    transposition -= 12;
                                    self.track.set_transposition(transposition);
                                }

                                ui.add_space(8.0);
                            });
                        });

                        ui.separator();

                        let mut ignore_ch10 = self.track.ignore_ch10();
                        if ui
                            .checkbox(&mut ignore_ch10, "Skip ch.10 (drums)")
                            .changed()
                        {
                            self.track.set_ignore_ch10(ignore_ch10);
                        }
                    });
                })
            })
            .response
    }
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
