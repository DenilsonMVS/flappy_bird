
use nalgebra_glm as glm;

use crate::{Atlas, AtlasFrame, graphics::renderer::{fonts::TextRenderConfig, positioning::{BaseDimensions, PositionMode, scale_dimension}, simple_texture::SimpleTexture}};

const DEFAULT_BUTTON_HEIGHT: f32 = 0.2;
const DEFAULT_BUTTON_PADDING: f32 = 0.04;

pub struct Button {
    content: &'static str,
    center: glm::Vec2,
    dimensions: glm::Vec2,
    content_color: glm::U8Vec3,
    padding: f32,
    mouse_inside: bool,
    mouse_enter_event_ack: bool,
}

impl Button {
    pub fn new(center: glm::Vec2, content: &'static str, content_color: glm::U8Vec3, atlas: &SimpleTexture<Atlas>) -> Self {
        let (_, original_size) = atlas.get_frame_info(AtlasFrame::Button);
        let width = scale_dimension(original_size, BaseDimensions::Height(DEFAULT_BUTTON_HEIGHT));
        return Self {
            center,
            dimensions: glm::vec2(width, DEFAULT_BUTTON_HEIGHT),
            mouse_inside: false, mouse_enter_event_ack: false,
            content,
            padding: DEFAULT_BUTTON_PADDING,
            content_color
        };
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

    pub fn get_text_render_config(&self) -> TextRenderConfig<'static> {
        TextRenderConfig {
            text: self.content,
            position: self.center,
            line_height: self.dimensions.y - self.padding * 2.0,
            position_mode: PositionMode::Center,
            color: self.content_color,
        }
    }

    pub fn is_mouse_inside(&self) -> bool {
        self.mouse_inside
    }

    fn is_mouse_position_inside(&self, mouse_position: glm::Vec2) -> bool {
        mouse_position.x >= self.center.x - self.dimensions.x * 0.5 &&
        mouse_position.x <= self.center.x + self.dimensions.x * 0.5 &&
        mouse_position.y >= self.center.y - self.dimensions.y * 0.5 &&
        mouse_position.y <= self.center.y + self.dimensions.y * 0.5
    }
}

