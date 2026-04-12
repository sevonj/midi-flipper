mod panels;
mod session;
mod toast;
mod util;
mod widgets;

use std::fmt::format;
use std::path::PathBuf;

use eframe::App;
use eframe::CreationContext;
use egui::CentralPanel;
use egui::Ui;
use egui_toast::Toasts;

use crate::app::session::Session;
use crate::app::widgets::SessionView;

#[derive(Default)]
pub struct MidiFlipperApp {
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
        
        let Some(file_path) = util::save_midi_file(file_name.to_str().unwrap_or_default()) else {
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
