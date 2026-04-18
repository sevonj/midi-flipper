use egui::Panel;
use egui::Ui;

use crate::MidiFlipperApp;
use crate::app::AppTab;
use crate::app::widgets::Tab;

impl MidiFlipperApp {
    pub(crate) fn tabs_bar(&mut self, ui: &mut Ui) -> egui::Response {
        Panel::top("tabs_bar")
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    Tab::value(
                        ui,
                        &mut self.tab,
                        AppTab::Session,
                        &AppTab::Session.to_string(),
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
            })
            .response
    }
}
