// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::RangeInclusive;

use egui::Slider;
use egui::Widget;

pub struct TranspositionControl<'a> {
    transposition: &'a mut i32,
}

impl<'a> TranspositionControl<'a> {
    pub fn new(transposition: &'a mut i32) -> Self {
        Self { transposition }
    }
}

impl Widget for TranspositionControl<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut response = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;

                if ui
                    .add(
                        Slider::new(self.transposition, -127..=127)
                            .smart_aim(false)
                            .custom_formatter(format_transposition),
                    )
                    .on_hover_text("Transposition")
                    .changed()
                {
                    changed = true;
                }

                if ui.button("+1").on_hover_text("Semitone Up").clicked() {
                    *self.transposition += 1;
                    changed = true;
                }
                if ui.button("-1").on_hover_text("Semitone Down").clicked() {
                    *self.transposition -= 1;
                    changed = true;
                }
                if ui.button("+Oct").on_hover_text("Octave Up").clicked() {
                    *self.transposition += 12;
                    changed = true;
                }
                if ui.button("-Oct").on_hover_text("Octave Down").clicked() {
                    *self.transposition -= 12;
                    changed = true;
                }

                ui.add_space(4.0);
            })
            .response;

        if changed {
            response.mark_changed();
        }

        response
    }
}

fn format_transposition(value: f64, _range: RangeInclusive<usize>) -> String {
    let transposition = value as i32;
    let sign = match transposition {
        x if x > 0 => "+",
        x if x < 0 => "-",
        _ => "",
    };

    let octaves = transposition.abs() / 12;
    let oct_string = if octaves != 0 {
        format!("{octaves}:")
    } else {
        String::new()
    };
    let semitones = transposition.abs() % 12;
    format!("{sign}{oct_string}{semitones}")
}
