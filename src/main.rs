// SPDX-License-Identifier: AGPL-3.0-or-later

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Midi Flipper Pro",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx
                .options_mut(|opt| opt.zoom_with_keyboard = false);
            Ok(Box::new(midi_flipper::MidiFlipperApp::new(cc)))
        }),
    )
}
