#[derive(Debug, Clone)]
pub struct FlipSettings {
    pub enabled: bool,
    pub global_center: u8,
    pub transpose: i32,
    pub ignore_ch10: bool,
}

impl FlipSettings {
    pub fn new(global_center: u8) -> Self {
        Self {
            enabled: true,
            global_center,
            transpose: 0,
            ignore_ch10: true,
        }
    }
}
