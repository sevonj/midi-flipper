use std::hash::Hash;

use egui::Frame;
use egui::Label;
use egui::RichText;
use egui::Sense;
use egui::Shadow;
use egui::Stroke;
use egui::Ui;
use egui::UiBuilder;
use egui::Widget;

pub struct Tab<'a> {
    text: &'a str,
    is_current: bool,
    id: egui::Id,
}

impl<'a> Tab<'a> {
    pub fn new(text: &'a str, is_current: bool, id_salt: impl Hash) -> Self {
        Self {
            text,
            is_current,
            id: egui::Id::new(id_salt),
        }
    }

    pub fn value<Value: PartialEq>(
        ui: &mut Ui,
        current_value: &mut Value,
        tab_value: Value,
        text: &'a str,
        id_salt: impl Hash,
    ) -> egui::Response {
        let mut response = ui.add(Self::new(text, *current_value == tab_value, id_salt));
        if response.clicked() && *current_value != tab_value {
            *current_value = tab_value;
            response.mark_changed();
        }
        response
    }
}

impl Widget for Tab<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.style_mut().spacing.item_spacing.x = 1.0;
        let sense = Sense::union(Sense::click(), Sense::hover());

        ui.scope_builder(UiBuilder::new().id(self.id).sense(sense), |ui| {
            let style = (*ui.ctx().global_style()).clone();
            let response = ui.response();
            let fill = if self.is_current {
                style.interact(&response).bg_fill
            //} else if response.hovered() {
            //    style.interact(&response).weak_bg_fill
            } else {
                style.visuals.faint_bg_color
            };
            let shadow = Shadow {
                offset: [0, 2],
                color: if self.is_current {
                    style.visuals.selection.bg_fill
                } else {
                    fill
                },
                ..Default::default()
            };
            Frame::group(&style)
                .inner_margin(4.)
                .outer_margin(0.)
                .corner_radius(0.)
                .stroke(Stroke::NONE)
                .fill(fill)
                .shadow(shadow)
                .show(ui, |ui| {
                    ui.style_mut().spacing.item_spacing.x = 0.0;
                    ui.add_space(4.0);
                    ui.add(
                        Label::new(
                            RichText::new(self.text).color(style.interact(&response).text_color()),
                        )
                        .selectable(false),
                    );

                    ui.add_space(6.0);

                    //let close_symbol = "❌";
                    //if ui
                    //    .add(Button::new(RichText::new(close_symbol).size(14.0)).frame(false))
                    //    .on_hover_text("Close this playlist")
                    //    .clicked()
                    //{
                    //    let _ = player.remove_playlist(index);
                    //}
                    ui.add_space(2.0);
                });
            response
        })
        .inner
    }
}
