use egui::CentralPanel;
use egui::Widget;

pub struct StatusPage<'a> {
    title: &'a str,
    subtitle: &'a str,
}

impl<'a> StatusPage<'a> {
    pub fn new(title: &'a str, subtitle: &'a str) -> Self {
        Self { title, subtitle }
    }
}

impl Widget for StatusPage<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        CentralPanel::default()
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(self.title);
                    ui.label(self.subtitle);
                })
                .response
            })
            .response
    }
}
