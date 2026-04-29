// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Button;
use egui::Color32;
use egui::Frame;
use egui::Image;
use egui::Layout;
use egui::Panel;
use egui::Slider;
use egui::Ui;
use egui::vec2;

use crate::MidiFlipperApp;
use crate::app::AppTab;
use crate::app::shortcuts::SHORTCUT_FILE_CLOSE;
use crate::app::shortcuts::SHORTCUT_FILE_OPEN;
use crate::app::shortcuts::SHORTCUT_FILE_SAVE;
use crate::app::shortcuts::SHORTCUT_PLAYBACK_PAUSE;
use crate::app::shortcuts::SHORTCUT_PLAYBACK_PLAYSTOP;
use crate::app::shortcuts::SHORTCUT_QUIT;
use crate::app::widgets::GlobalControls;
use crate::app::widgets::PlaybackControls;
use crate::app::widgets::Tab;

const BAR_HEIGHT: f32 = 48.0;

impl MidiFlipperApp {
    pub(crate) fn menu_bar(&mut self, ui: &mut Ui) -> egui::Response {
        Panel::top("menu_bar")
            .resizable(false)
            .exact_size(BAR_HEIGHT)
            .show_separator_line(false)
            .frame(
                Frame::default()
                    .inner_margin(vec2(8., 2.))
                    .fill(Color32::from_hex("#4a4a4a").unwrap()),
            )
            .show_inside(ui, |ui| {
                let bar_rect = ui.content_rect();
                Image::from(egui::include_image!(
                    "../../../assets/tex_toolbar_gradient.svg"
                ))
                .paint_at(ui, bar_rect.with_max_y(bar_rect.min.y + BAR_HEIGHT));

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            self.file_menu(ui);
                            self.playback_menu(ui);
                        });

                        ui.with_layout(Layout::left_to_right(egui::Align::Max), |ui| {
                            Tab::value(
                                ui,
                                &mut self.tab,
                                AppTab::Tracks,
                                &AppTab::Tracks.to_string(),
                                "tab_session",
                            );
                            Tab::value(
                                ui,
                                &mut self.tab,
                                AppTab::Log,
                                &AppTab::Log.to_string(),
                                "tab_log",
                            );
                        });
                    });

                    ui.separator();

                    ui.add(PlaybackControls::new(&mut self.session));

                    ui.separator();

                    ui.add(GlobalControls::new(&mut self.session));

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.label("Master Volume");

                        let state_id = ui.id().with("tracks_timeline_state");

                        let mut volume = self.master_volume();
                        let mut use_big_range =
                            ui.data_mut(|d| d.get_temp::<bool>(state_id).unwrap_or_default());

                        ui.horizontal(|ui| {
                            let loud_changed = ui.checkbox(&mut use_big_range, "⚠ Loud").changed();
                            let range = if use_big_range { 0.0..=10.0 } else { 0.0..=1.5 };
                            if ui.add(Slider::new(&mut volume, range)).changed() || loud_changed {
                                self.set_master_volume(volume);
                            }
                        });

                        ui.data_mut(|d| d.insert_temp(state_id, use_big_range));
                    });
                });
            })
            .response
    }

    fn file_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui
                .add(
                    Button::new("Open")
                        .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_FILE_OPEN)),
                )
                .clicked()
            {
                self.prompt_open_midi();
            }

            if ui
                .add_enabled(
                    self.can_save(),
                    Button::new("Save")
                        .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_FILE_SAVE)),
                )
                .clicked()
            {
                self.prompt_save_file();
            }

            if ui
                .add_enabled(
                    self.is_session_open(),
                    Button::new("Close")
                        .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_FILE_CLOSE)),
                )
                .clicked()
            {
                self.close_session();
            }

            ui.separator();

            if ui
                .add(Button::new("Quit").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_QUIT)))
                .clicked()
            {
                ui.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn playback_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Playback", |ui| {
            let has_session = !self.session.is_placeholder();

            if ui
                .add_enabled(
                    has_session && !self.session.is_playing(),
                    Button::new("Play")
                        .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_PLAYBACK_PLAYSTOP)),
                )
                .clicked()
            {
                self.session.play();
            }

            if ui
                .add_enabled(
                    has_session && self.session.is_playing(),
                    Button::new("Pause")
                        .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_PLAYBACK_PAUSE)),
                )
                .clicked()
            {
                self.session.pause();
            }

            let mut stop_button = Button::new("Stop");
            if !self.session.is_playback_in_progress() || self.session.is_playing() {
                stop_button = stop_button
                    .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_PLAYBACK_PLAYSTOP));
            }
            if ui
                .add_enabled(
                    has_session && self.session.is_playback_in_progress(),
                    stop_button,
                )
                .clicked()
            {
                self.session.stop();
            }

            ui.separator();

            ui.label("Soundfont");
            let has_custom_sf = self.custom_soundfont().is_some();
            if ui.radio(!has_custom_sf, "Default").clicked() {
                self.set_custom_soundfont(None);
                self.log_text(String::from("Loaded default soundfont"));
            };

            let custom_sf_label = if let Some(sf) = self.custom_soundfont() {
                sf.get_info().get_bank_name()
            } else {
                "Use Custom"
            };
            if ui.radio(has_custom_sf, custom_sf_label).clicked() {
                self.prompt_open_soundfont();
            };
        });
    }
}
