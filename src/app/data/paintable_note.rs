// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Vec2;
use egui::vec2;

#[derive(Debug, Clone, Copy)]
pub struct PaintableNote {
    points: [Vec2; 2],
}

impl PaintableNote {
    pub fn new(note: u8, start: f32, end: f32) -> Self {
        let y = (127 - note) as f32 / 128.0;
        Self {
            points: [vec2(start, y), vec2(end, y)],
        }
    }

    pub fn points(&self) -> &[Vec2; 2] {
        &self.points
    }
}
