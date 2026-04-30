// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Align;
use egui::Button;
use egui::Image;
use egui::Layout;
use egui::Modal;
use egui::OpenUrl;
use egui::RichText;
use egui::Ui;
use egui::include_image;
use egui::vec2;

use crate::MidiFlipperApp;
use crate::app::widgets::ActionRow;

const WIDTH: f32 = 320.0;

impl MidiFlipperApp {
    pub(crate) fn about_dialog(&mut self, ui: &mut Ui) {
        let modal = Modal::new("bout".into());
        if modal
            .show(ui.ctx(), |ui| {
                ui.set_width(WIDTH);

                title_bar(ui);

                info_self(ui);
            })
            .should_close()
        {
            self.show_about = false;
        };
    }
}

fn title_bar(ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        Button::image(include_image!("../../../assets/icon_close.svg"))
                            .image_tint_follows_text_color(true)
                            .frame(false),
                    )
                    .on_hover_text("Close")
                    .clicked()
                {
                    ui.close();
                }
            });
        });
    });
}

fn info_self(ui: &mut Ui) {
    ui.add_space(32.);

    ui.horizontal(|ui| {
        ui.add_space(62.);
        ui.add(
            Image::new(include_image!("../../../assets/icon_midiflipper.svg"))
                .fit_to_exact_size(vec2(64., 64.)),
        );
        ui.vertical(|ui| {
            ui.add_space(4.);
            ui.heading(RichText::new("Midi Flipper Pro").strong());
            ui.label("by Sevonj");
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        });
    });

    ui.add_space(32.);

    if ui
        .add(ActionRow::new(
            "Website",
            Some("Bug reports, feature requests, source code"),
            "about_website",
        ))
        .clicked()
    {
        ui.ctx()
            .open_url(OpenUrl::new_tab(env!("CARGO_PKG_REPOSITORY")));
    }
}
