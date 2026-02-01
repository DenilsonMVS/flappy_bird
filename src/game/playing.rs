
use std::time::{self, Duration};

use crate::{Atlas, AtlasFrame, MainContext, NextScene, game::{defs::{BIRD_RADIUS, BIRD_START_POSITION, GRAVITY, HORIZONTAL_SPEED, MAX_PIPE_CENTER_DIST, OFFSET_PIPE_DEL, PIPE_AMOUNT, PIPE_SPACE, PIPE_START_POSITION, PIPE_WIDTH, REPEATED_PIPES, SCENE_HEIGHT, SPACE_BETWEEN_PIPES, SPEED_GAIN_JUMPING}, scene::Scene}, graphics::renderer::{ClearField, atlas::UvInfo, fonts::{Font, TextRenderConfig}, positioning::{BaseDimensions, PositionMode, SimpleTransform, scale_dimension}, simple_texture::SimpleTexture}};
use glfw::{Action, Key};
use nalgebra_glm as glm;
use rand::{Rng, rngs::ThreadRng};
use rodio::Source;

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

    fn get_texture_state(&self, simple_texture: &SimpleTexture<Atlas>) -> UvInfo {
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

pub struct Playing {
    position: glm::Vec2,
    vertical_speed: f32,
    bird_fly_state: BirdFlyState,
    last_frame_time: std::time::Instant,
    time_to_change_scene: Option<std::time::Instant>,
    pipes: [Pipe; PIPE_AMOUNT],
    smaller_pipe: usize,
    rng: ThreadRng,
}

impl Playing {
    pub fn new() -> Self {
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
            last_frame_time: time::Instant::now(),
            pipes,
            smaller_pipe: 0,
            rng,
            time_to_change_scene: None
        };
    }

    fn calculate_score(&self) -> usize {
        ((self.position.x - PIPE_START_POSITION) / SPACE_BETWEEN_PIPES).max(0.0) as usize
    }

    fn send_text(&mut self, font: &mut Font) {
        let score = self.calculate_score();
        let score_text= format!("Score: {}", score);
        font.add_text(&TextRenderConfig {
            position: glm::vec2(1.0, -1.0),
            text: &score_text,
            color: glm::vec3(30u8, 15u8, 80u8),
            line_height: 0.25,
            position_mode: PositionMode::BottomRight,
        });
    }

    fn send_scene(&mut self, simple_texture: &mut SimpleTexture<Atlas>) {
        let scene_uv = simple_texture.get_frame_info(AtlasFrame::Scene);
        let scene_original_dimensions = scene_uv.get_original_dimensions();
        let scene_width = scale_dimension(scene_original_dimensions, BaseDimensions::Height(SCENE_HEIGHT));
        let scene_start_position = (-self.position.x * 0.5).rem_euclid(scene_width) - scene_width * 3.0;

        for offset in 0..5 {
            let scene_pos = scene_start_position + scene_width * offset as f32;
            simple_texture.add_quad(
                glm::vec2(scene_pos, 1.0),
                PositionMode::TopLeft,
                BaseDimensions::Height(SCENE_HEIGHT),
                &scene_uv,
            );
        }
    }

    fn send_pipes(&mut self, simple_texture: &mut SimpleTexture<Atlas>) {
        let pipe_entrance_uv = simple_texture.get_frame_info(AtlasFrame::TopCano);
        let pipe_entrance_original_dimensions = pipe_entrance_uv.get_original_dimensions();
        let pipe_uv = simple_texture.get_frame_info(AtlasFrame::Cano);
        let pipe_original_dimensions = pipe_uv.get_original_dimensions();
        let pipe_entrance_height = scale_dimension(pipe_entrance_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));
        let pipe_height = scale_dimension(pipe_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));

        for &pipe in self.pipes.iter() {
            let position = pipe.x_position - self.position.x;

            {
                let pipe_start_pos = pipe.opening_y_position - PIPE_SPACE * 0.5;

                simple_texture.add_quad(
                    glm::vec2(position, pipe_start_pos),
                    PositionMode::TopCenter,
                    BaseDimensions::Width(PIPE_WIDTH),
                    &pipe_entrance_uv
                );

                for idx in 0..REPEATED_PIPES {
                    let pipe_y_pos = pipe_start_pos - pipe_entrance_height - pipe_height * idx as f32;
                    simple_texture.add_quad(
                        glm::vec2(position, pipe_y_pos),
                        PositionMode::TopCenter,
                        BaseDimensions::Width(PIPE_WIDTH),
                        &pipe_uv
                    );
                }
            }
            
            {
                let pipe_start_pos = pipe.opening_y_position + PIPE_SPACE * 0.5;

                simple_texture.add_quad_simple_transform(
                    glm::vec2(position, pipe.opening_y_position + PIPE_SPACE * 0.5),
                    PositionMode::BottomCenter,
                    BaseDimensions::Width(PIPE_WIDTH),
                    &pipe_entrance_uv,
                    SimpleTransform::FlipVertical
                );

                for idx in 0..REPEATED_PIPES {
                    let pipe_y_pos = pipe_start_pos + pipe_entrance_height + pipe_height * idx as f32;
                    simple_texture.add_quad_simple_transform(
                        glm::vec2(position, pipe_y_pos),
                        PositionMode::BottomCenter,
                        BaseDimensions::Width(PIPE_WIDTH),
                        &pipe_uv,
                        SimpleTransform::FlipVertical
                    );
                }
            }
        }
    }

    fn send_bird(&mut self, simple_texture: &mut SimpleTexture<Atlas>) {
        let uv_data = self.bird_fly_state.get_texture_state(simple_texture);
        let original_size = uv_data.get_original_dimensions();
        
        let velocity = glm::vec2(HORIZONTAL_SPEED, self.vertical_speed);
        let direction = glm::normalize(&velocity);
        let up_vector = glm::vec2(-direction.y, direction.x);

        simple_texture.add_oriented_quad(
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

impl Scene for Playing {
    fn handle_input(&mut self, context: &mut MainContext) {
        context.glfw.poll_events();

        if self.time_to_change_scene.is_some() {
            return;
        }

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
        if let Some(end_time) = self.time_to_change_scene {
            if *now > end_time {
                *context.next_scene = Some(NextScene::MainMenu);
            }
            return;
        }

        if self.position.x - self.pipes[self.smaller_pipe].x_position > OFFSET_PIPE_DEL {
            let prev_pipe = &self.pipes[self.smaller_pipe.wrapping_sub(1) % PIPE_AMOUNT];
            self.pipes[self.smaller_pipe] = Pipe {
                x_position: prev_pipe.x_position + SPACE_BETWEEN_PIPES,
                opening_y_position: self.rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            };
            self.smaller_pipe += 1;
        }        
        
        if self.check_collisions() {
            self.time_to_change_scene = Some(*now + Duration::from_secs(3));
            context.sound_library.sound_player.play(context.sound_library.death.get().amplify(10.0));
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
        let simple_texture = &mut context.texture_library.simple_texture;
        let font = &mut context.font_library.deja_vu_sans;

        self.send_text(font);
        font.send();

        self.send_scene(simple_texture);
        self.send_pipes(simple_texture);
        self.send_bird(simple_texture);

        simple_texture.send();

        renderer.clear(&[ClearField::Color]);
        context.texture_library.simple_texture_renderer.draw(projection_matrix, simple_texture);
        context.font_library.fonts.draw(font, projection_matrix);
    }
}
