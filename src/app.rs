mod panels;
mod session;
mod toast;
mod widgets;

use std::path::PathBuf;

use eframe::App;
use eframe::CreationContext;
use egui::Ui;
use egui_toast::Toasts;
use rfd::FileDialog;

use crate::app::session::Session;
use crate::app::widgets::SessionView;

#[derive(Default)]
pub struct MidiFlipperApp {
    workdir: Option<PathBuf>,
    session: Option<Session>,
    toasts: Toasts,
}

impl MidiFlipperApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }

    pub fn try_open_file(&mut self, file_path: PathBuf) {
        self.workdir = file_path.parent().map(|p| p.to_path_buf());

        let session = match Session::from_file(file_path) {
            Ok(session) => session,
            Err(e) => {
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
        match std::fs::write(file_path, bytes) {
            Ok(_) => self.toast_success("Saved!"),
            Err(e) => self.toast_err(e.to_string()),
        }
    }

    pub fn close_session(&mut self) {
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
}

impl App for MidiFlipperApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.top_panel(ui);

        let Some(session) = &mut self.session else {
            ui.add(widgets::StatusPage::new(
                "Nothing Open",
                "Open a midi file from the file menu.",
            ));
            return;
        };

        ui.add(SessionView::new(session));

        self.toasts.show(ui);
    }
}
