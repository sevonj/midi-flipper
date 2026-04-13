mod panels;
mod session;
mod widgets;

use std::collections::VecDeque;
use std::path::PathBuf;
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
use egui_toast::Toasts;
use rfd::FileDialog;

use crate::MidiFlipperError;
use crate::app::session::Session;
use crate::app::widgets::LogView;
use crate::app::widgets::SessionView;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    #[default]
    Main,
    Log,
}

impl std::fmt::Display for AppTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppTab::Main => write!(f, "Main"),
            AppTab::Log => write!(f, "Log"),
        }
    }
}

pub struct MidiFlipperApp {
    workdir: Option<PathBuf>,
    session: Option<Session>,
    toasts: Toasts,
    log: VecDeque<String>,
    tab: AppTab,
    start: Instant,
}

impl Default for MidiFlipperApp {
    fn default() -> Self {
        Self {
            workdir: Default::default(),
            session: Default::default(),
            toasts: Default::default(),
            log: Default::default(),
            tab: Default::default(),
            start: Instant::now(),
        }
    }
}

impl MidiFlipperApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }

    pub fn try_open_file(&mut self, file_path: PathBuf) {
        self.log_text(format!("Opening {file_path:?}"));
        self.workdir = file_path.parent().map(|p| p.to_path_buf());

        let session = match Session::from_file(file_path) {
            Ok(session) => session,
            Err(e) => {
                self.log_err(&e);
                self.toast_err(e.to_string());
                return;
            }
        };
        self.session = Some(session);
    }

    pub fn export_midi(&mut self) {
        let Some(session) = &self.session else {
            return;
        };
        let Some(flipped_midi) = session.flipped_midi() else {
            return;
        };

        let session_name = PathBuf::from(session.name());
        let stem = session_name
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let file_name = PathBuf::from(format!("{stem}_flip")).with_extension("mid");
        let Some(file_path) = self.save_midi_file(file_name.to_str().unwrap_or_default()) else {
            return;
        };
        let bytes = flipped_midi.to_midi();

        self.log_text(format!("Exporting into {file_path:?}"));

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
        self.log_text(format!("Closing session"));
        self.session = None;
    }

    fn pick_midi_file(&self) -> Option<PathBuf> {
        let mut dialog = FileDialog::new().add_filter("MIDI Files", &["mid", "midi"]);
        if let Some(dir) = &self.workdir {
            dialog = dialog.set_directory(dir);
        }
        dialog.pick_file()
    }

    fn save_midi_file(&self, file_name: &str) -> Option<PathBuf> {
        let mut dialog = FileDialog::new()
            .add_filter("MIDI Files", &["mid"])
            .set_file_name(file_name);
        if let Some(dir) = &self.workdir {
            dialog = dialog.set_directory(dir);
        }
        dialog.save_file()
    }

    fn toast_success(&mut self, text: impl Into<WidgetText>) {
        self.toasts.add(
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
        self.toasts.add(
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
        let text = match e {
            MidiFlipperError::Io(e) => e.to_string(),
            MidiFlipperError::MidiParse(e) => e.to_string(),
            MidiFlipperError::OutputValidationFailed => String::from(
                "Output validation failed. Probably because of this: https://github.com/AlexCharlton/midi-msg/issues/32",
            ),
        };
        self.log_text(text)
    }

    fn log_text(&mut self, text: String) {
        while self.log.len() > 99 {
            self.log.pop_front();
        }
        self.log.push_back(text);
    }

    fn tab_session(&mut self, ui: &mut Ui) {
        let Some(session) = &mut self.session else {
            ui.add(widgets::StatusPage::new(
                "Nothing Open",
                "Open a midi file from the file menu.",
            ));
            return;
        };
        ui.add(SessionView::new(session));
    }

    fn tab_log(&mut self, ui: &mut Ui) {
        ui.add(LogView::new(&self.log));
    }
}

impl App for MidiFlipperApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        install_image_loaders(ui.ctx());

        if Instant::now() - self.start < Duration::from_secs(3) {
            const SPLASH_SIZE: Vec2 = Vec2 { x: 400.0, y: 300.0 };
            ui.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            ui.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
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
        ui.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ui.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
        ui.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(640.0, 480.0)));

        self.top_panel(ui);
        self.bottom_panel(ui);

        match self.tab {
            AppTab::Main => self.tab_session(ui),
            AppTab::Log => self.tab_log(ui),
        }

        self.toasts.show(ui);
    }
}
