
use nalgebra_glm::{self as glm};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameInfo {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl FrameInfo {
    pub fn get_uv(&self) -> UvInfo {
        let min = glm::vec2(self.x, self.y);
        let frame_size = glm::vec2(self.width, self.height);
        let max = min + frame_size;

        return UvInfo { min, max };
    }

    pub fn get_original_dimensions(&self) -> glm::Vec2 {
        glm::vec2(self.width as f32, self.height as f32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UvInfo {
    pub min: glm::U16Vec2,
    pub max: glm::U16Vec2,
}

pub trait TypedAtlas: Sized {
    type Frame: Copy;
    fn new(bytes: &[u8]) -> Result<Self>;
    fn get_info(&self, frame: Self::Frame) -> FrameInfo;
    fn dimensions(&self) -> nalgebra_glm::U16Vec2;
}
