// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Button;
use egui::Frame;
use egui::Image;
use egui::Label;
use egui::Layout;
use egui::RichText;
use egui::Vec2;
use egui::Widget;
use egui::include_image;

use crate::app::data::Session;
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
                .inner_margin(4.0)
                .fill(fill)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.set_height(ui.available_height());

                    ui.vertical(|ui| {
                        ui.set_width(192.0);

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

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.spacing_mut().item_spacing.x = 2.0;

                        ui.vertical(|ui| {
                            let original = session.playback_original();
                            if ui
                                .selectable_label(!original, RichText::new("FLIP").monospace())
                                .clicked()
                            {
                                session.set_playback_original(!original);
                            }
                            if ui
                                .selectable_label(original, RichText::new("ORIG").monospace())
                                .clicked()
                            {
                                session.set_playback_original(!original);
                            }
                        });
                        ui.image(include_image!("../../../assets/icon_flipswap.svg"));
                    });
                })
        })
        .response
    }
}
