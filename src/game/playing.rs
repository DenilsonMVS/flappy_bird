
use std::time;

use crate::{MainContext, game::scene::Scene, graphics::renderer::{BlendFactor, Capability, ClearField, Renderer, atlas::{FrameInfo, TypedAtlas, UvInfo}, positioning::{BaseDimensions, PositionMode, SimpleTransform, scale_dimension}, simple_texture::{SimpleTexture, SimpleTextureRenderer}, texture::{MagFiltering, MinFiltering, TextureWrap}}};
use glfw::{Action, Key};
use nalgebra_glm as glm;
use rand::{Rng, rngs::ThreadRng};
use vertex_derive::atlas_bundle;

const HORIZONTAL_SPEED: f32 = 0.5;
const SCENE_HEIGHT: f32 = 2.4;
const BIRD_START_POSITION: f32 = -0.8;
const PIPE_AMOUNT: usize = 16;
const PIPE_SPACE: f32 = 0.6;
const SPACE_BETWEEN_PIPES: f32 = 1.5;
const PIPE_START_POSITION: f32 = 1.0;
const MAX_PIPE_CENTER_DIST: f32 = 0.6;
const PIPE_WIDTH: f32 = 0.3;
const REPEATED_PIPES: usize = 5;
const GRAVITY: f32 = 4.0;
const OFFSET_PIPE_DEL: f32 = 8.0;
const BIRD_RADIUS: f32 = 0.075;

struct BirdFlyState {
    current_index: u8,
    last_change: std::time::Instant,
}

impl BirdFlyState {
    fn new() -> Self {
        Self {
            current_index: 0,
            last_change: std::time::Instant::now(),
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

    fn get_texture_state(&self, simple_texture: &SimpleTexture<Atlas>) -> (UvInfo, glm::Vec2) {
        simple_texture.get_frame_info(match self.current_index {
            0 => AtlasFrame::Bird1,
            1 | 4 => AtlasFrame::Bird2,
            2 => AtlasFrame::Bird3,
            3 => AtlasFrame::Bird4,
            _ => unreachable!()
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Pipe {
    x_position: f32,
    opening_y_position: f32,
}

#[atlas_bundle("res/textures/atlas/texture.json")]
struct Atlas;

pub struct Playing<'a> {
    position: glm::Vec2,
    vertical_speed: f32,
    simple_texture_renderer: SimpleTextureRenderer<'a>,
    simple_texture: SimpleTexture<'a, Atlas>,
    bird_fly_state: BirdFlyState,
    last_frame_time: std::time::Instant,
    pipes: [Pipe; PIPE_AMOUNT],
    smaller_pipe: usize,
    rng: ThreadRng,
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
            include_bytes!("../../res/textures/atlas/texture.json")
        ).unwrap();
        
        let mut rng = rand::rng();
        let pipes: [Pipe; PIPE_AMOUNT] = std::array::from_fn(|idx| {
            Pipe {
                x_position: PIPE_START_POSITION + (idx as f32) * SPACE_BETWEEN_PIPES,
                opening_y_position: rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            }
        });

        return Self {
            position: glm::vec2(BIRD_START_POSITION, 0.0),
            vertical_speed: 0.5,
            bird_fly_state: BirdFlyState::new(),
            simple_texture_renderer,
            simple_texture,
            last_frame_time: time::Instant::now(),
            pipes,
            smaller_pipe: 0,
            rng
        };
    }
}

impl<'a> Scene for Playing<'a> {
    fn handle_input(&mut self, context: &mut MainContext) {
        const SPEED_GAIN_JUMPING: f32 = 2.5;

        context.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&context.events) {
            match event {
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    self.vertical_speed += SPEED_GAIN_JUMPING;
                    self.bird_fly_state.on_jump(&context.now);

                    context.sound_library.sound_player.play(context.sound_library.jump.get());
                }
                _ => (),
            }
        }
    }
    
    fn game_logic(&mut self, context: &mut MainContext) {
        let now = &context.now;

        if self.position.x - self.pipes[self.smaller_pipe].x_position > OFFSET_PIPE_DEL {
            let prev_pipe = &self.pipes[self.smaller_pipe.wrapping_sub(1) % PIPE_AMOUNT];
            self.pipes[self.smaller_pipe] = Pipe {
                x_position: prev_pipe.x_position + SPACE_BETWEEN_PIPES,
                opening_y_position: self.rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            };
            self.smaller_pipe += 1;
        }

        
        if self.check_collisions() {
            context.window.set_should_close(true);
        }


        self.bird_fly_state.update(now);

        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = *now;

        self.vertical_speed -= GRAVITY * delta;
        self.position += glm::vec2(HORIZONTAL_SPEED, self.vertical_speed) * delta;
    }

    fn generate_output(&mut self, context: &mut MainContext) {
        let renderer = context.renderer;
        let projection_matrix = &context.proj_matrix;

        self.draw_scene();
        self.draw_pipes();
        self.draw_bird();

        self.simple_texture.send();

        renderer.clear(&[ClearField::Color]);
        self.simple_texture_renderer.draw(projection_matrix, &mut self.simple_texture);
    }
}

impl<'a> Playing<'a> {
    fn draw_scene(&mut self) {
        let (scene_uv, scene_original_dimensions) = self.simple_texture.get_frame_info(AtlasFrame::Scene);
        let scene_width = scale_dimension(scene_original_dimensions, BaseDimensions::Height(SCENE_HEIGHT));
        let scene_start_position = (-self.position.x * 0.5).rem_euclid(scene_width) - scene_width * 3.0;

        for offset in 0..5 {
            let scene_pos = scene_start_position + scene_width * offset as f32;
            self.simple_texture.add_quad(
                glm::vec2(scene_pos, 1.0),
                PositionMode::TopLeft,
                scene_original_dimensions,
                BaseDimensions::Height(SCENE_HEIGHT),
                &scene_uv,
            );
        }
    }

    fn draw_pipes(&mut self) {
        let (pipe_entrance_uv, pipe_entrance_original_dimensions) = self.simple_texture.get_frame_info(AtlasFrame::TopCano);
        let (pipe_uv, pipe_original_dimensions) = self.simple_texture.get_frame_info(AtlasFrame::Cano);
        let pipe_entrance_height = scale_dimension(pipe_entrance_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));
        let pipe_height = scale_dimension(pipe_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));

        for &pipe in self.pipes.iter() {
            let position = pipe.x_position - self.position.x;

            {
                let pipe_start_pos = pipe.opening_y_position - PIPE_SPACE * 0.5;

                self.simple_texture.add_quad(
                    glm::vec2(position, pipe_start_pos),
                    PositionMode::TopCenter,
                    pipe_entrance_original_dimensions,
                    BaseDimensions::Width(PIPE_WIDTH),
                    &pipe_entrance_uv
                );

                for idx in 0..REPEATED_PIPES {
                    let pipe_y_pos = pipe_start_pos - pipe_entrance_height - pipe_height * idx as f32;
                    self.simple_texture.add_quad(
                        glm::vec2(position, pipe_y_pos),
                        PositionMode::TopCenter,
                        pipe_original_dimensions,
                        BaseDimensions::Width(PIPE_WIDTH),
                        &pipe_uv
                    );
                }
            }
            
            {
                let pipe_start_pos = pipe.opening_y_position + PIPE_SPACE * 0.5;

                self.simple_texture.add_quad_simple_transform(
                    glm::vec2(position, pipe.opening_y_position + PIPE_SPACE * 0.5),
                    PositionMode::BottomCenter,
                    pipe_entrance_original_dimensions,
                    BaseDimensions::Width(PIPE_WIDTH),
                    &pipe_entrance_uv,
                    SimpleTransform::FlipVertical
                );

                for idx in 0..REPEATED_PIPES {
                    let pipe_y_pos = pipe_start_pos + pipe_entrance_height + pipe_height * idx as f32;
                    self.simple_texture.add_quad_simple_transform(
                        glm::vec2(position, pipe_y_pos),
                        PositionMode::BottomCenter,
                        pipe_original_dimensions,
                        BaseDimensions::Width(PIPE_WIDTH),
                        &pipe_uv,
                        SimpleTransform::FlipVertical
                    );
                }
            }
        }
    }

    fn draw_bird(&mut self) {
        let (uv_data, original_size) = self.bird_fly_state.get_texture_state(&self.simple_texture);
        
        let velocity = glm::vec2(HORIZONTAL_SPEED, self.vertical_speed);
        let direction = glm::normalize(&velocity);
        let up_vector = glm::vec2(-direction.y, direction.x);

        self.simple_texture.add_oriented_quad(
            glm::vec2(BIRD_START_POSITION, self.position.y),
            PositionMode::Center,
            original_size,
            BaseDimensions::Height(BIRD_RADIUS * 2.0),
            up_vector,
            &uv_data,
        );
    }

    fn check_collisions(&self) -> bool {
        let bird_world_x = self.position.x + BIRD_START_POSITION;
        let bird_center = glm::vec2(bird_world_x, self.position.y);
    
        return self.position.y < -1.0 || self.position.y > 1.0 ||
            self.pipes.iter().fold(false, |prev, pipe|
                prev || {
                    let pipe_left = pipe.x_position - PIPE_WIDTH * 0.5;
                    let pipe_right = pipe.x_position + PIPE_WIDTH * 0.5;

                    let gap_bottom = pipe.opening_y_position - PIPE_SPACE * 0.5;
                    let gap_top = pipe.opening_y_position + PIPE_SPACE * 0.5;

                    return Self::circle_rect_collision(
                        bird_center, BIRD_RADIUS,
                        glm::vec2(pipe_left, -10.0),
                        glm::vec2(pipe_right, gap_bottom),
                    ) || Self::circle_rect_collision(
                        bird_center, BIRD_RADIUS,
                        glm::vec2(pipe_left, gap_top),
                        glm::vec2(pipe_right, 10.0),
                    );
                }
        );
    }

    fn circle_rect_collision(circle_center: glm::Vec2, radius: f32, rect_min: glm::Vec2, rect_max: glm::Vec2) -> bool {
        let closest_point = glm::vec2(
            circle_center.x.clamp(rect_min.x, rect_max.x),
            circle_center.y.clamp(rect_min.y, rect_max.y)
        );

        let distance = glm::distance(&circle_center, &closest_point);
        return distance < radius;
    }
}