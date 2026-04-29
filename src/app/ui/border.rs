// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Color32;
use egui::Frame;
use egui::Image;
use egui::ImageSource;
use egui::Panel;
use egui::Pos2;
use egui::Rect;
use egui::Stroke;
use egui::TextureOptions;
use egui::Ui;
use egui::Vec2;
use egui::include_image;
use egui::pos2;

use crate::MidiFlipperApp;

const WIDTH: f32 = 12.0;
const TEX_WOULD: ImageSource<'_> = include_image!("../../../assets/tex_border.png");
const TEX_TOP: ImageSource<'_> = include_image!("../../../assets/tex_border_top.svg");
const CORNER_RADIUS: f32 = 1.0;

impl MidiFlipperApp {
    pub(crate) fn border(&mut self, ui: &mut Ui) {
        let stroke = Stroke::new(1.5, Color32::BLACK);

        Panel::left("border_left")
            .resizable(false)
            .exact_size(WIDTH)
            .frame(Frame::NONE)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                image(ui, Pos2::ZERO);

                let rect = ui.max_rect();
                let b = rect.max;
                let a = pos2(b.x, rect.min.y);
                ui.painter().line(vec![a, b], stroke);
            });

        Panel::right("border_right")
            .resizable(false)
            .exact_size(WIDTH)
            .frame(Frame::NONE)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                image(ui, pos2(0.5, 0.0));

                let rect = ui.max_rect();
                let a = rect.min;
                let b = pos2(a.x, rect.max.y);
                ui.painter().line(vec![a, b], stroke);
            });
    }
}

fn image(ui: &mut Ui, uv_pos: Pos2) {
    let rect = ui.max_rect();
    let uv = Rect::from_min_size(uv_pos, rect.size() / Vec2::splat(2048.0));

    Image::from(TEX_WOULD)
        .texture_options(TextureOptions::LINEAR_REPEAT)
        .maintain_aspect_ratio(false)
        .corner_radius(CORNER_RADIUS)
        .uv(uv)
        .paint_at(ui, rect);

    Image::from(TEX_TOP)
        .maintain_aspect_ratio(true)
        .corner_radius(CORNER_RADIUS)
        .paint_at(ui, rect.with_max_y(rect.min.y + 32.0));
}
