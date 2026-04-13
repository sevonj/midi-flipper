use egui::Panel;
use egui::Ui;

use crate::MidiFlipperApp;
use crate::app::AppTab;

impl MidiFlipperApp {
    pub(crate) fn bottom_panel(&mut self, ui: &mut Ui) -> egui::Response {
        Panel::bottom("bottom_panel")
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.tab,
                        crate::app::AppTab::Main,
                        AppTab::Main.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.tab,
                        crate::app::AppTab::Log,
                        AppTab::Log.to_string(),
                    );
                });
            })
            .response
    }
}
