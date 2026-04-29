// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Button;
use egui::Color32;
use egui::Frame;
use egui::Image;
use egui::Label;
use egui::Layout;
use egui::Rect;
use egui::RichText;
use egui::Sense;
use egui::UiBuilder;
use egui::Vec2;
use egui::Widget;
use egui::include_image;
use egui::pos2;
use egui::vec2;

use crate::app::data::Session;
use crate::app::shortcuts::SHORTCUT_PLAYBACK_TOGGLE_FLIP;
use crate::util::format_duration;

pub struct PlaybackControls<'a> {
    session: &'a mut Session,
}

impl<'a> PlaybackControls<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session }
    }
}

impl Widget for PlaybackControls<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let session = self.session;

        let is_playing = session.is_playing();
        let is_playback_in_progress = session.is_playback_in_progress();
        let icon_pp = if is_playing {
            include_image!("../../../assets/tex_button_pause.svg")
        } else if is_playback_in_progress {
            include_image!("../../../assets/tex_button_resume.svg")
        } else {
            include_image!("../../../assets/tex_button_play.svg")
        };

        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            if session.is_placeholder() {
                ui.disable();
            }

            let fill: egui::Color32 = ui.style().visuals.widgets.active.bg_fill;
            Frame::new()
                .inner_margin(0.0)
                .fill(fill)
                .corner_radius(100.0)
                .show(ui, |ui| {
                    ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(0.0);

                        ui.add_space(3.0);

                        if ui
                            .add(
                                Button::new(Image::from(icon_pp).fit_to_original_size(0.8))
                                    .frame(false)
                                    .image_tint_follows_text_color(true),
                            )
                            .clicked()
                        {
                            if !is_playing {
                                session.play();
                            } else {
                                session.pause();
                            }
                        }

                        if ui
                            .add(
                                Button::new(
                                    Image::from(include_image!(
                                        "../../../assets/tex_button_stop.svg"
                                    ))
                                    .fit_to_original_size(0.8),
                                )
                                .frame(false)
                                .image_tint_follows_text_color(true),
                            )
                            .clicked()
                        {
                            session.stop();
                        }

                        ui.add_space(2.0);
                    });
                });

            Frame::new()
                .inner_margin(0.0)
                .fill(fill)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.set_height(ui.available_height());

                    Frame::new().inner_margin(4.0).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(170.0);

                            ui.add(
                                Label::new(RichText::new(session.name()).monospace())
                                    .wrap_mode(egui::TextWrapMode::Truncate),
                            );
                            ui.horizontal(|ui| {
                                let position = if session.is_playback_in_progress() {
                                    format_duration(session.playback_position())
                                } else {
                                    String::from("--:--")
                                };
                                let duration = if !session.is_placeholder() {
                                    format_duration(session.playback_duration())
                                } else {
                                    String::from("--:--")
                                };
                                ui.monospace(format!("{position}/{duration}"));
                            })
                        });
                    });

                    let mut flipbox_response = ui
                        .scope_builder(
                            UiBuilder::new()
                                .id_salt("flip_toggle")
                                .sense(Sense::click()),
                            |ui| {
                                let response = ui.response();

                                let fill = if response.hovered() {
                                    ui.style().visuals.widgets.hovered.bg_fill
                                } else {
                                    Color32::TRANSPARENT
                                };

                                Frame::new()
                                    .outer_margin(1)
                                    .fill(fill)
                                    .corner_radius(3.0)
                                    .show(ui, |ui| {
                                        ui.set_height(ui.available_height());

                                        ui.horizontal(|ui| {
                                            ui.set_height(ui.available_height());
                                            ui.spacing_mut().item_spacing.y = 0.0;
                                            ui.spacing_mut().item_spacing.x = 2.0;

                                            ui.add_space(2.0);

                                            ui.vertical(|ui| {
                                                let style = ui.style();

                                                let frame_weak = Frame::new()
                                                    .inner_margin(2.)
                                                    .outer_margin(0.)
                                                    .corner_radius(2.);

                                                let frame_stronk = Frame::new()
                                                    .fill(style.visuals.selection.bg_fill)
                                                    .inner_margin(2.)
                                                    .outer_margin(0.)
                                                    .corner_radius(2.);

                                                ui.add_space(2.0);

                                                if session.playback_original() {
                                                    frame_weak.show(ui, |ui| {
                                                        ui.add(
                                                            Label::new(
                                                                RichText::new("FLIP").monospace(),
                                                            )
                                                            .selectable(false),
                                                        );
                                                    });
                                                    frame_stronk.show(ui, |ui| {
                                                        ui.add(
                                                            Label::new(
                                                                RichText::new("ORIG")
                                                                    .monospace()
                                                                    .strong(),
                                                            )
                                                            .selectable(false),
                                                        );
                                                    });
                                                } else {
                                                    frame_stronk.show(ui, |ui| {
                                                        ui.add(
                                                            Label::new(
                                                                RichText::new("FLIP")
                                                                    .monospace()
                                                                    .strong(),
                                                            )
                                                            .selectable(false),
                                                        );
                                                    });
                                                    frame_weak.show(ui, |ui| {
                                                        ui.add(
                                                            Label::new(
                                                                RichText::new("ORIG").monospace(),
                                                            )
                                                            .selectable(false),
                                                        );
                                                    });
                                                }
                                            });

                                            let img_x = ui.next_widget_position().x;
                                            ui.add_space(32.0);

                                            let img_pos = pos2(img_x, 0.0);

                                            Image::from(include_image!(
                                                "../../../assets/icon_flipswap.svg"
                                            ))
                                            .paint_at(
                                                ui,
                                                Rect::from_min_size(img_pos, vec2(32.0, 48.0)),
                                            );
                                        });
                                    });
                            },
                        )
                        .response;

                    flipbox_response = if session.playback_original() {
                        flipbox_response.on_hover_text("Switch to flipped (F)")
                    } else {
                        flipbox_response.on_hover_text("Switch to original (F)")
                    };

                    if ui.is_enabled()
                        && (flipbox_response.clicked()
                            || ui.input_mut(|input| {
                                input.consume_shortcut(&SHORTCUT_PLAYBACK_TOGGLE_FLIP)
                            }))
                    {
                        session.set_playback_original(!session.playback_original());
                    }
                })
        })
        .response
    }
}
