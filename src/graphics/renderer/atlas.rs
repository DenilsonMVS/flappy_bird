
use std::collections::HashMap;

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



pub struct Atlas {
    dimensions: glm::U32Vec2,
    frames: HashMap<String, FrameInfo>,
}

impl Atlas {
    pub fn new(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let frames: HashMap<String, FrameInfo> = serde_json::from_slice(bytes)?;
        let dimensions = frames.values()
            .fold(glm::vec2(0u32, 0u32), |prev, frame|
                glm::vec2(
                    prev.x.max(frame.x + frame.width),
                    prev.y.max(frame.y + frame.height)
                )
            );
        return Ok(Self { frames, dimensions });
    }

    pub fn get_frame_info(&self, frame: &str) -> Option<&FrameInfo> {
        self.frames.get(frame)
    }

    pub fn get_uv_info(&self, frame: &str) -> Option<UvInfo> {
        self.frames.get(frame).map(|frame| frame.to_uv(&self.dimensions))
    }

    pub fn get_dimensions(&self) -> glm::U32Vec2 {
        self.dimensions
    }
}
