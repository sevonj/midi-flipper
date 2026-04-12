use std::path::PathBuf;

use rfd::FileDialog;

pub(crate) fn pick_midi_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("MIDI Files", &["mid", "midi"])
        .pick_file()
}
pub(crate) fn save_midi_file(file_name: &str) -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("MIDI Files", &["mid"])
        .set_file_name(file_name)
        .save_file()
}
