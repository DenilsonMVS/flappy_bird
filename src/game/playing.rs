
use std::time;

use crate::{game::scene::Scene, graphics::renderer::{BlendFactor, Capability, ClearField, Renderer, atlas::UvInfo, positioning::{BaseDimensions, PositionMode}, simple_texture::{SimpleTexture, SimpleTextureRenderer}, texture::{MagFiltering, MinFiltering, TextureWrap}}};
use glfw::{Action, Glfw, GlfwReceiver, Key, WindowEvent};
use nalgebra_glm as glm;

struct BirdFlyState {
    current_index: u8,
    last_change: std::time::Instant,
    original_size: glm::Vec2,
    texture_states: [UvInfo; 4],
}

impl BirdFlyState {
    fn new(simple_texture: &SimpleTexture) -> Self {
        let bird1 = simple_texture.get_frame_info("bird_1").unwrap();
        let bird2 = simple_texture.get_frame_info("bird_2").unwrap();
        let bird3 = simple_texture.get_frame_info("bird_3").unwrap();
        let bird4 = simple_texture.get_frame_info("bird_4").unwrap();

        Self {
            current_index: 0,
            last_change: std::time::Instant::now(),
            texture_states: [
                bird1.0,
                bird2.0,
                bird3.0,
                bird4.0,
            ],
            original_size: bird1.1
        }
    }

    fn update(&mut self, now: &std::time::Instant) {
        if self.current_index == 0 {
            return;
        }

        let delta = now.duration_since(self.last_change);
        if delta.as_millis() > 100 {
            self.current_index = (self.current_index + 1) % 5;
            self.last_change = *now;
        }
    }

    fn on_jump(&mut self, now: &std::time::Instant) {
        self.current_index = 1;
        self.last_change = *now;
    }

    fn get_original_size(&self) -> glm::Vec2 {
        self.original_size
    }

    fn get_texture_state(&self) -> &UvInfo {
        &self.texture_states[match self.current_index {
            0 => 0,
            1 | 4 => 1,
            2 => 2,
            3 => 3,
            _ => unreachable!()
        }]
    }
}

pub struct Playing<'a> {
    position: glm::Vec2,
    vertical_speed: f32,
    simple_texture_renderer: SimpleTextureRenderer<'a>,
    simple_texture: SimpleTexture<'a>,
    bird_fly_state: BirdFlyState,
    last_frame_time: std::time::Instant,
}

impl<'a> Playing<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        renderer.enable(Capability::Blend);
        renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        renderer.clear_color(&glm::Vec4::new(0.1, 0.2, 0.3, 0.0));

        let simple_texture_renderer = SimpleTextureRenderer::new(renderer);
        let simple_texture = SimpleTexture::new(
            renderer,
            include_bytes!("../../res/textures/atlas/texture.png"),
            MagFiltering::Nearest, MinFiltering::Nearest, TextureWrap::ClampToBorder,
            include_bytes!("../../res/textures/atlas/texture.json"),
        ).unwrap();

        return Self {
            position: glm::vec2(-0.8, 0.0),
            vertical_speed: 0.5,
            bird_fly_state: BirdFlyState::new(&simple_texture),
            simple_texture_renderer,
            simple_texture,
            last_frame_time: time::Instant::now(),
        };
    }
}

const HORIZONTAL_SPEED: f32 = 0.5;

impl<'a> Scene for Playing<'a> {
    fn handle_input(&mut self,
        glfw: &mut Glfw,
        events: &mut GlfwReceiver<(f64, WindowEvent)>,
        now: &std::time::Instant,
    ) {
        const SPEED_GAIN_JUMPING: f32 = 3.0;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    self.vertical_speed += SPEED_GAIN_JUMPING;
                    self.bird_fly_state.on_jump(now);
                }
                _ => (),
            }
        }
    }
    
    fn game_logic(&mut self, now: &std::time::Instant) {
        const GRAVITY: f32 = 3.0;

        self.bird_fly_state.update(now);

        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = *now;

        self.vertical_speed -= GRAVITY * delta;
        self.position += glm::vec2(HORIZONTAL_SPEED, self.vertical_speed) * delta;
    }

    fn generate_output(&mut self, renderer: &Renderer, projection_matrix: &glm::Mat4) {
        let uv_data = self.bird_fly_state.get_texture_state();
        let original_size = self.bird_fly_state.get_original_size();
        
        let velocity = glm::vec2(HORIZONTAL_SPEED, self.vertical_speed);
        let direction = glm::normalize(&velocity);
        let up_vector = glm::vec2(-direction.y, direction.x);

        self.simple_texture.add_oriented_quad(
            self.position,
            PositionMode::Center,
            original_size,
            BaseDimensions::Height(0.15),
            up_vector,
            uv_data);

        self.simple_texture.send();

        renderer.clear(&[ClearField::Color]);
        self.simple_texture_renderer.draw(projection_matrix, &mut self.simple_texture);
    }
}