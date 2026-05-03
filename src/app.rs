// SPDX-License-Identifier: AGPL-3.0-or-later

mod data;
mod shortcuts;
mod ui;
mod widgets;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use eframe::App;
use eframe::CreationContext;
use egui::Image;
use egui::Layout;
use egui::Ui;
use egui::Vec2;
use egui::WidgetText;
use egui_extras::install_image_loaders;
use egui_toast::Toast;
use egui_toast::ToastKind;
use egui_toast::ToastOptions;
use rfd::FileDialog;
use rustysynth::SoundFont;

use crate::MidiFlipperError;
use crate::app::data::AppSettings;
use crate::app::data::AppState;
use crate::app::data::Session;
use crate::app::widgets::LogView;
use crate::app::widgets::Timeline;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    #[default]
    Tracks,
    Log,
}

impl std::fmt::Display for AppTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppTab::Tracks => write!(f, "Tracks"),
            AppTab::Log => write!(f, "Log"),
        }
    }
}

#[derive(Default, Debug)]
struct ModalState {
    pub show_about: bool,
    pub show_about_legal: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MidiFlipperApp {
    #[serde(skip)]
    session: Session,
    settings: AppSettings,
    #[serde(skip)]
    state: AppState,
}

impl Default for MidiFlipperApp {
    fn default() -> Self {
        let mut this = Self {
            session: Session::placeholder(),
            settings: Default::default(),
            state: Default::default(),
        };
        this.log_text(String::from("Hello there!"));
        this
    }
}

impl MidiFlipperApp {
    pub const fn master_volume(&self) -> f32 {
        self.state.master_volume
    }

    pub fn set_master_volume(&mut self, master_volume: f32) {
        self.state.master_volume = master_volume;
        self.session.set_playback_volume(master_volume);
    }

    pub fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx
            .set_fonts(epaint_ubuntu_fonts::font_definitions());
        cc.egui_ctx.set_theme(eframe::egui::Theme::Dark);
        let mut this: MidiFlipperApp = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        if let Some(file_path) = this.settings.custom_soundfont_path.take() {
            this.try_open_soundfont(file_path);
        }

        this
    }

    pub fn prompt_open_midi(&mut self) {
        let Some(file_path) = self.pick_midi_file() else {
            return;
        };
        self.try_open_midi(file_path);
    }

    pub fn prompt_open_soundfont(&mut self) {
        let Some(file_path) = self.pick_soundfont() else {
            return;
        };
        self.try_open_soundfont(file_path);
    }

    pub fn try_open_midi(&mut self, file_path: PathBuf) {
        if let Err(e) = self.open_midi(file_path) {
            self.log_err(&e);
            self.toast_err(e.to_string());
        }
    }

    pub fn open_midi(&mut self, file_path: PathBuf) -> Result<(), MidiFlipperError> {
        self.log_text(format!("Opening {file_path:?}"));
        self.settings.workdir = file_path.parent().map(|p| p.to_path_buf());
        let mut session = Session::from_file(file_path)?;
        session.set_custom_soundfont(self.state.custom_soundfont.clone());
        self.state.session_init = false;
        session.set_playback_volume(self.state.master_volume);
        self.session = session;
        Ok(())
    }

    pub fn try_open_soundfont(&mut self, file_path: PathBuf) {
        if let Err(e) = self.open_soundfont(file_path) {
            self.log_err(&e);
            self.toast_err(e.to_string());
        }
    }

    pub fn open_soundfont(&mut self, file_path: PathBuf) -> Result<(), MidiFlipperError> {
        self.log_text(format!("Opening {file_path:?}"));
        let file = std::fs::File::open(&file_path)?;
        let soundfont = SoundFont::new(&mut std::io::BufReader::new(file))?;
        let sf_name = soundfont.get_info().get_bank_name().to_string();
        self.settings.custom_soundfont_path = Some(file_path);
        self.state.custom_soundfont = Some(std::sync::Arc::new(soundfont));
        self.session
            .set_custom_soundfont(self.custom_soundfont().clone());

        self.log_text(format!("Loaded soundfont: {sf_name:?}"));
        Ok(())
    }

    pub fn is_session_open(&self) -> bool {
        !self.session.is_placeholder()
    }

    pub fn can_save(&self) -> bool {
        self.is_session_open()
    }

    pub fn custom_soundfont(&self) -> &Option<Arc<SoundFont>> {
        &self.state.custom_soundfont
    }

    pub fn clear_custom_soundfont(&mut self) {
        self.settings.custom_soundfont_path = None;
        self.state.custom_soundfont = None;
        self.session.set_custom_soundfont(None);
    }

    pub fn prompt_save_file(&mut self) {
        if !self.can_save() {
            return;
        }
        let flipped_midi = self.session.assemble_flipped_midi();

        let session_name = PathBuf::from(self.session.name());
        let stem = session_name
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let file_name = PathBuf::from(format!("{stem}_flip")).with_extension("mid");
        let Some(file_path) = self.save_midi_file(file_name.to_str().unwrap_or_default()) else {
            return;
        };
        let bytes = flipped_midi.to_midi();

        self.log_text(format!("Saving into {file_path:?}"));

        match std::fs::write(file_path, bytes) {
            Ok(_) => self.toast_success("Saved!"),
            Err(e) => {
                let e = e.into();
                self.log_err(&e);
                self.toast_err(e.to_string())
            }
        }
    }

    pub fn close_session(&mut self) {
        if !self.is_session_open() {
            return;
        }
        self.log_text("Closing session".to_string());
        self.session = Session::placeholder();
    }

    fn pick_midi_file(&self) -> Option<PathBuf> {
        let mut dialog = FileDialog::new().add_filter("MIDI Files", &["mid", "midi"]);
        if let Some(dir) = &self.settings.workdir {
            dialog = dialog.set_directory(dir);
        }
        dialog.pick_file()
    }

    fn pick_soundfont(&self) -> Option<PathBuf> {
        let mut dialog = FileDialog::new().add_filter("Soundfont", &["sf2"]);
        if let Some(dir) = &self.settings.workdir {
            dialog = dialog.set_directory(dir);
        }
        dialog.pick_file()
    }

    fn save_midi_file(&self, file_name: &str) -> Option<PathBuf> {
        let mut dialog = FileDialog::new()
            .add_filter("MIDI Files", &["mid"])
            .set_file_name(file_name);
        if let Some(dir) = &self.settings.workdir {
            dialog = dialog.set_directory(dir);
        }
        dialog.save_file()
    }

    fn toast_success(&mut self, text: impl Into<WidgetText>) {
        self.state.toasts.add(
            Toast::new()
                .text(text)
                .options(
                    ToastOptions::default()
                        .show_progress(true)
                        .duration_in_seconds(5.),
                )
                .kind(ToastKind::Success),
        );
    }

    fn toast_err(&mut self, text: impl Into<WidgetText>) {
        self.state.toasts.add(
            Toast::new()
                .text(text)
                .options(
                    ToastOptions::default()
                        .show_progress(true)
                        .duration_in_seconds(5.),
                )
                .kind(ToastKind::Error),
        );
    }

    fn log_err(&mut self, e: &MidiFlipperError) {
        self.log_text(e.to_string())
    }

    fn log_text(&mut self, text: String) {
        while self.state.log.len() > 99 {
            self.state.log.pop_front();
        }
        self.state.log.push_back(text);
    }

    fn tab_tracks(&mut self, ui: &mut Ui) {
        ui.add(Timeline::new(&mut self.session));
    }

    fn tab_log(&mut self, ui: &mut Ui) {
        ui.add(LogView::new(&self.state.log));
    }
}

impl App for MidiFlipperApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // --- Setup
        install_image_loaders(ui.ctx());

        // --- Amazin Professoinal Enterprise Quality Splash Screen
        if Instant::now() - self.state.start < Duration::from_secs(3) {
            const SPLASH_SIZE: Vec2 = Vec2 { x: 400.0, y: 300.0 };
            ui.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            ui.send_viewport_cmd(egui::ViewportCommand::InnerSize(SPLASH_SIZE));
            Image::from(egui::include_image!("../assets/bootsplash.png"))
                .paint_at(ui, ui.content_rect());
            ui.with_layout(Layout::bottom_up(egui::Align::Max), |ui| {
                ui.monospace(format!(
                    "2026 — MAXIMUM MIDI TWISTER 360 {}",
                    env!("CARGO_PKG_VERSION")
                ));
            });
            return;
        }
        if !self.state.splash_done {
            ui.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
            ui.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(1200.0, 800.0)));
            self.state.splash_done = true;
        }

        if !self.state.session_init {
            Timeline::reset_zoom_position(ui);
            self.state.session_init = true;
        }

        // --- Actual UI
        self.menu_bar(ui);
        ui.add_space(1.0);

        if self.settings.border {
            self.border(ui);
        }

        match self.state.tab {
            AppTab::Tracks => self.tab_tracks(ui),
            AppTab::Log => self.tab_log(ui),
        }

        if self.state.modal_state.show_about {
            self.about_dialog(ui);
        }
        if self.state.modal_state.show_about_legal {
            self.about_legal_dialog(ui);
        }

        self.consume_shortcuts(ui);
        self.state.toasts.show(ui);
        self.session.check_for_changes();
        if self.session.is_playing() {
            ui.request_repaint();
        }

        ui.input(|i| {
            let Some(file) = i.raw.dropped_files.first() else {
                return;
            };
            let Some(file_path) = file.path.clone() else {
                return;
            };
            let Some(ext) = file_path.extension() else {
                return;
            };

            match ext.to_ascii_lowercase().to_str() {
                Some("mid") | Some("midi") => self.try_open_midi(file_path),
                Some("sf2") => self.try_open_soundfont(file_path),
                _ => (),
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_after_session_change() {
        const VOL: f32 = 0.69;
        let mut app = MidiFlipperApp::default();

        app.set_master_volume(VOL);
        assert_eq!(app.master_volume(), VOL);

        app.open_midi(PathBuf::from("samples/icream.mid")).unwrap();
        assert_eq!(app.master_volume(), VOL);
        assert_eq!(app.session.playback_volume(), VOL);

        app.open_midi(PathBuf::from("samples/icream.mid")).unwrap();
        assert_eq!(app.master_volume(), VOL);
        assert_eq!(app.session.playback_volume(), VOL);

        app.close_session();
        app.open_midi(PathBuf::from("samples/icream.mid")).unwrap();
        assert_eq!(app.master_volume(), VOL);
        assert_eq!(app.session.playback_volume(), VOL)
    }
}
