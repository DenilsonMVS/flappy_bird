
use nalgebra_glm as glm;

use crate::{game::main_menu::{Atlas, AtlasFrame}, graphics::renderer::{positioning::{BaseDimensions, PositionMode, scale_dimension}, simple_texture::SimpleTexture}};

pub struct Button {
    center: glm::Vec2,
    dimensions: glm::Vec2,
    mouse_inside: bool,
    mouse_enter_event_ack: bool,
}

impl Button {
    pub fn new(center: glm::Vec2, height: f32, atlas: &SimpleTexture<Atlas>) -> Self {
        let (_, original_size) = atlas.get_frame_info(AtlasFrame::Button);
        let width = scale_dimension(original_size, BaseDimensions::Height(height));
        Self { center, dimensions: glm::vec2(width, height), mouse_inside: false, mouse_enter_event_ack: false }
    }

    pub fn update(&mut self, mouse_position: glm::Vec2) {
        let mouse_inside = self.is_mouse_position_inside(mouse_position);

        if !self.mouse_inside && mouse_inside {
            self.mouse_enter_event_ack = false;
        }
        
        self.mouse_inside = mouse_inside;
    }

    pub fn submit_data(&self, atlas: &mut SimpleTexture<Atlas>) {
        let (uv_data_idle, _) = atlas.get_frame_info(AtlasFrame::Button);
        let (uv_data_hover, original_size) = atlas.get_frame_info(AtlasFrame::HoverButton);
        let uv_data = match self.mouse_inside {
            false => uv_data_idle,
            true => uv_data_hover
        };

        atlas.add_quad(
            self.center,
            PositionMode::Center,
            original_size,
            BaseDimensions::Height(self.dimensions.y),
            &uv_data,
        );
    }

    pub fn did_mouse_enter(&mut self) -> bool {
        if self.mouse_inside && !self.mouse_enter_event_ack {
            self.mouse_enter_event_ack = true;
            return true;
        } else {
            return false;
        }
    }

    fn is_mouse_position_inside(&self, mouse_position: glm::Vec2) -> bool {
        mouse_position.x >= self.center.x - self.dimensions.x * 0.5 &&
        mouse_position.x <= self.center.x + self.dimensions.x * 0.5 &&
        mouse_position.y >= self.center.y - self.dimensions.y * 0.5 &&
        mouse_position.y <= self.center.y + self.dimensions.y * 0.5
    }
}

