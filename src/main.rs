fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Midi Flipper Pro",
        native_options,
        Box::new(|cc| Ok(Box::new(midi_flipper::MidiFlipperApp::new(cc)))),
    )
}
