// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppSettings {
    pub workdir: Option<PathBuf>,
    pub custom_soundfont_path: Option<PathBuf>,
    pub border: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            workdir: None,
            custom_soundfont_path: None,
            border: false,
        }
    }
}
