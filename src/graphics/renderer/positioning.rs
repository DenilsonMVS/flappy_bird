
use nalgebra_glm as glm;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionMode {
	TopLeft,    TopCenter,    TopRight,
	CenterLeft, Center,       CenterRight,
	BottomLeft, BottomCenter, BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub enum BaseDimensions {
    Width(f32),
    Height(f32),
}

pub struct Box {
    pub min: glm::Vec2,
    pub max: glm::Vec2,
}

pub fn generate_box(
    position: glm::Vec2,
    position_mode: PositionMode,
    original_size: glm::Vec2,
    base_dimension: BaseDimensions,
) -> Box {
    let aspect_ratio = original_size.x / original_size.y;
    
    let size = match base_dimension {
        BaseDimensions::Width(w) => glm::vec2(w, w / aspect_ratio),
        BaseDimensions::Height(h) => glm::vec2(h * aspect_ratio, h),
    };

    let offset = match position_mode {
        PositionMode::TopLeft      => glm::vec2(0.0, -size.y),
        PositionMode::TopCenter    => glm::vec2(-size.x / 2.0, -size.y),
        PositionMode::TopRight     => glm::vec2(-size.x, -size.y),
        
        PositionMode::CenterLeft   => glm::vec2(0.0, -size.y / 2.0),
        PositionMode::Center       => glm::vec2(-size.x / 2.0, -size.y / 2.0),
        PositionMode::CenterRight  => glm::vec2(-size.x, -size.y / 2.0),
        
        PositionMode::BottomLeft   => glm::vec2(0.0, 0.0),
        PositionMode::BottomCenter => glm::vec2(-size.x / 2.0, 0.0),
        PositionMode::BottomRight  => glm::vec2(-size.x, 0.0),
    };

    let min = position + offset;
    let max = min + size;

    return Box { min, max };
}

pub struct OrientedBox {
    pub top_left: glm::Vec2,
    pub top_right: glm::Vec2,
    pub bot_left: glm::Vec2,
    pub bot_right: glm::Vec2,
}

pub fn generate_oriented_box(
    position: glm::Vec2,
    position_mode: PositionMode,
    original_size: glm::Vec2,
    base_dimension: BaseDimensions,
    up_vector: glm::Vec2,
) -> OrientedBox {
    let up = glm::normalize(&up_vector);
    let right = glm::vec2(up.y, -up.x);

    let aspect_ratio = original_size.x / original_size.y;
    let size = match base_dimension {
        BaseDimensions::Width(w) => glm::vec2(w, w / aspect_ratio),
        BaseDimensions::Height(h) => glm::vec2(h * aspect_ratio, h),
    };

    let half_w = size.x / 2.0;
    let half_h = size.y / 2.0;

    let center_offset = match position_mode {
        PositionMode::TopLeft      => right * half_w - up * half_h,
        PositionMode::TopCenter    => -up * half_h,
        PositionMode::TopRight     => -right * half_w - up * half_h,
        PositionMode::CenterLeft   => right * half_w,
        PositionMode::Center       => glm::vec2(0.0, 0.0),
        PositionMode::CenterRight  => -right * half_w,
        PositionMode::BottomLeft   => right * half_w + up * half_h,
        PositionMode::BottomCenter => up * half_h,
        PositionMode::BottomRight  => -right * half_w + up * half_h,
    };

    let center = position + center_offset;

    let half_up = up * half_h;
    let half_right = right * half_w;

    return OrientedBox {
        top_left:  center + half_up - half_right,
        top_right: center + half_up + half_right,
        bot_left:  center - half_up - half_right,
        bot_right: center - half_up + half_right,
    };
}
