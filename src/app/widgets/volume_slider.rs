// SPDX-License-Identifier: AGPL-3.0-or-later

use core::ops::RangeInclusive;
use std::vec;

use egui::Color32;
use egui::Painter;
use egui::Slider;
use egui::Stroke;
use egui::Widget;
use egui::pos2;
use egui::vec2;

pub struct VolumeSlider<'a> {
    volume: &'a mut f32,
    range: RangeInclusive<f32>,
    rms: Option<(f32, f32)>,
}

impl<'a> VolumeSlider<'a> {
    pub fn new(volume: &'a mut f32) -> Self {
        Self {
            volume,
            range: 0.0..=1.5,
            rms: None,
        }
    }

    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    pub fn rms(mut self, rms: (f32, f32)) -> Self {
        self.rms = Some(rms);
        self
    }
}

impl Widget for VolumeSlider<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .add(Slider::new(self.volume, self.range.clone()))
            .on_hover_text("Volume");

        if let Some((rms_l, rms_r)) = self.rms {
            let rect = response.rect;
            let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), rect);

            let a_l = pos2(rect.min.x + 4.0, rect.min.y + rect.height() / 2.0 - 2.0);
            let a_r = pos2(rect.min.x + 4.0, rect.min.y + rect.height() / 2.0 + 2.0);
            let b_off = vec2(rect.size().x - 8.0, 0.0);
            let range_len = self.range.end() - self.range.start();
            painter.line(
                vec![a_l, a_l + b_off * rms_l / range_len * *self.volume],
                Stroke::new(2.0, Color32::GREEN),
            );
            painter.line(
                vec![a_r, a_r + b_off * rms_r / range_len * *self.volume],
                Stroke::new(2.0, Color32::GREEN),
            );
        }

        response
    }
}
