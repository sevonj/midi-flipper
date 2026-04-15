use egui::Frame;
use egui::Label;
use egui::RichText;
use egui::Widget;

use crate::app::session::SessionTrack;

const HEIGHT: f32 = 64.0;

pub struct TrackView<'a> {
    index: usize,
    track: &'a mut SessionTrack,
}

impl<'a> TrackView<'a> {
    pub fn new(index: usize, track: &'a mut SessionTrack) -> Self {
        Self { index, track }
    }
}

impl Widget for TrackView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let style = ui.style();
            let weak_bg_fill = style.visuals.widgets.open.weak_bg_fill;

            Frame::group(&style)
                .inner_margin(0.)
                .outer_margin(0.)
                .corner_radius(0.)
                .fill(weak_bg_fill)
                .show(ui, |ui| {
                    ui.set_height(HEIGHT);
                    ui.set_width(ui.available_width());

                    ui.horizontal_centered(|ui| {
                        ui.set_width(18.);
                        ui.add_space(4.);
                        ui.label(self.index.to_string());
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        track_name_label(ui, self.track);
                        ui.checkbox(self.track.flip_mut(), "Flip");
                    });
                })
        })
        .response
    }
}

fn track_name_label(ui: &mut egui::Ui, track: &mut SessionTrack) {
    let text = if !track.is_midi() {
        RichText::new("[unknown track type]").weak()
    } else if let Some(name) = track.name() {
        if name.is_empty() {
            RichText::new("[track name is empty]").weak()
        } else {
            RichText::new(name)
        }
    } else {
        RichText::new("[unnamed track]").weak()
    };
    ui.add(Label::new(text).wrap_mode(egui::TextWrapMode::Truncate));
}
