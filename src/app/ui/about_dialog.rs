// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Align;
use egui::Button;
use egui::Image;
use egui::Label;
use egui::Layout;
use egui::Modal;
use egui::OpenUrl;
use egui::RichText;
use egui::ScrollArea;
use egui::Ui;
use egui::include_image;
use egui::vec2;

use crate::MidiFlipperApp;
use crate::app::widgets::ActionRow;

const WIDTH: f32 = 320.0;

const LEGAL_TEXT: &str = include_str!("../../../generated_licenses.txt");

impl MidiFlipperApp {
    pub(crate) fn about_dialog(&mut self, ui: &mut Ui) {
        let modal = Modal::new("bout".into());
        if modal
            .show(ui.ctx(), |ui| {
                ui.set_width(WIDTH);

                title_bar("", ui);

                self.info_self(ui);
            })
            .should_close()
        {
            self.state.modal_state.show_about = false;
        };
    }

    pub(crate) fn about_legal_dialog(&mut self, ui: &mut Ui) {
        let modal = Modal::new("legal".into());
        if modal
            .show(ui.ctx(), |ui| {
                ui.set_min_height(ui.content_rect().height() - 100.0);

                title_bar("Licenses", ui);
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add(Label::new(LEGAL_TEXT).halign(Align::Center));
                })
            })
            .should_close()
        {
            self.state.modal_state.show_about_legal = false;
        };
    }

    fn info_self(&mut self, ui: &mut Ui) {
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

        if ui
            .add(ActionRow::new(
                "Legal",
                Some("Open source licenses"),
                "about_legal",
            ))
            .clicked()
        {
            self.state.modal_state.show_about_legal = true;
        }
    }
}

fn title_bar(title: &str, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.add(Label::new(RichText::new(title).heading()).selectable(false));
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
