
use nalgebra_glm::{self as glm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameInfo {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl FrameInfo {
    pub fn to_uv(&self, texture_dim: &glm::U32Vec2) -> UvInfo {
        let to_f32 = |x| x as f32;

        let min = glm::vec2(self.x, self.y);
        let frame_size = glm::vec2(self.width, self.height);
        let max = min + frame_size;

        let texture_dim = texture_dim.map(to_f32);

        return UvInfo {
            min: min.map(to_f32).component_div(&texture_dim),
            max: max.map(to_f32).component_div(&texture_dim),
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UvInfo {
    pub min: glm::Vec2,
    pub max: glm::Vec2,
}

pub trait TypedAtlas: Sized {
    type Frame: Copy;
    fn new(bytes: &[u8]) -> Option<Self>;
    fn get_info(&self, frame: Self::Frame) -> (UvInfo, nalgebra_glm::Vec2);
    fn dimensions(&self) -> nalgebra_glm::U32Vec2;
}
