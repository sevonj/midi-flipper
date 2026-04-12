use egui::WidgetText;
use egui_toast::Toast;
use egui_toast::ToastKind;

use crate::MidiFlipperApp;

impl MidiFlipperApp {
    pub(crate) fn toast_success(&mut self, text: impl Into<WidgetText>) {
        self.toasts
            .add(Toast::new().text(text).kind(ToastKind::Success));
    }

    pub(crate) fn toast_err(&mut self, text: impl Into<WidgetText>) {
        self.toasts
            .add(Toast::new().text(text).kind(ToastKind::Error));
    }
}
