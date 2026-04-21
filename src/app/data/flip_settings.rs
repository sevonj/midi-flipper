// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone)]
pub struct FlipSettings {
    pub enabled: bool,
    pub global_transpose: i32,
    pub global_flip_bend: bool,
    pub transpose: i32,
    pub ignore_ch10: bool,
}

impl FlipSettings {
    pub fn new(global_transpose: i32, global_flip_bend: bool) -> Self {
        Self {
            enabled: true,
            global_transpose,
            global_flip_bend,
            transpose: 0,
            ignore_ch10: true,
        }
    }
}
