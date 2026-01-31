use std::time;

use macros::atlas_bundle;
use rand::{Rng, rngs::ThreadRng};
use nalgebra_glm as glm;
use crate::{MainContext, game::{button::Button, defs::{BIRD_START_POSITION, HORIZONTAL_SPEED, MAX_PIPE_CENTER_DIST, OFFSET_PIPE_DEL, PIPE_AMOUNT, PIPE_SPACE, PIPE_START_POSITION, PIPE_WIDTH, REPEATED_PIPES, SCENE_HEIGHT, SPACE_BETWEEN_PIPES}, scene::Scene}, graphics::renderer::{BlendFactor, Capability, ClearField, Renderer, atlas::{FrameInfo, TypedAtlas, UvInfo}, positioning::{BaseDimensions, PositionMode, SimpleTransform, scale_dimension}, simple_texture::{SimpleTexture, SimpleTextureRenderer}, texture::{MagFiltering, MinFiltering, TextureWrap}}};

const PLAY_BUTTON_CENTER: glm::Vec2 = glm::Vec2::new(0.0, -0.2);
const QUIT_BUTTON_CENTER: glm::Vec2 = glm::Vec2::new(0.0, -0.5);
const BUTTON_HEIGHT: f32 = 0.2;

#[derive(Debug, Clone, Copy)]
struct Pipe {
    x_position: f32,
    opening_y_position: f32,
}

#[atlas_bundle("res/textures/atlas/texture.json")]
pub struct Atlas;

pub struct MainMenu<'a> {
    x_offset: f32,
    simple_texture_renderer: SimpleTextureRenderer<'a>,
    simple_texture: SimpleTexture<'a, Atlas>,
    pipes: [Pipe; PIPE_AMOUNT],
    smaller_pipe: usize,
    rng: ThreadRng,
    last_frame_time: std::time::Instant,
    buttons: [Button; 2],
}

impl<'a> MainMenu<'a> {
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
            buttons: [
                Button::new(PLAY_BUTTON_CENTER, BUTTON_HEIGHT, &simple_texture),
                Button::new(QUIT_BUTTON_CENTER, BUTTON_HEIGHT, &simple_texture),
            ],
            x_offset: BIRD_START_POSITION,
            simple_texture_renderer,
            simple_texture,
            pipes,
            smaller_pipe: 0,
            rng,
            last_frame_time: time::Instant::now(),
        };
    }

    fn send_scene(&mut self) {
        let (scene_uv, scene_original_dimensions) = self.simple_texture.get_frame_info(AtlasFrame::Scene);
        let scene_width = scale_dimension(scene_original_dimensions, BaseDimensions::Height(SCENE_HEIGHT));
        let scene_start_position = (-self.x_offset * 0.5).rem_euclid(scene_width) - scene_width * 3.0;

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

    fn send_pipes(&mut self) {
        let (pipe_entrance_uv, pipe_entrance_original_dimensions) = self.simple_texture.get_frame_info(AtlasFrame::TopCano);
        let (pipe_uv, pipe_original_dimensions) = self.simple_texture.get_frame_info(AtlasFrame::Cano);
        let pipe_entrance_height = scale_dimension(pipe_entrance_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));
        let pipe_height = scale_dimension(pipe_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));

        for &pipe in self.pipes.iter() {
            let position = pipe.x_position - self.x_offset;

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
}

impl<'a> Scene for MainMenu<'a> {
    fn handle_input(&mut self, context: &mut MainContext) {
        context.glfw.poll_events();
    }
    
    fn game_logic(&mut self, context: &mut MainContext) {
        if self.x_offset - self.pipes[self.smaller_pipe].x_position > OFFSET_PIPE_DEL {
            let prev_pipe = &self.pipes[self.smaller_pipe.wrapping_sub(1) % PIPE_AMOUNT];
            self.pipes[self.smaller_pipe] = Pipe {
                x_position: prev_pipe.x_position + SPACE_BETWEEN_PIPES,
                opening_y_position: self.rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            };
            self.smaller_pipe += 1;
        }

        for button in self.buttons.iter_mut() {
            button.update(context.mouse_pos);
        }

        let now = context.now;
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        self.x_offset += HORIZONTAL_SPEED * delta;
    }

    fn generate_output(&mut self, context: &mut MainContext) {
        let renderer = context.renderer;
        let projection_matrix = &context.proj_matrix;

        self.send_scene();
        self.send_pipes();

        for button in self.buttons.iter_mut() {
            button.submit_data(&mut self.simple_texture);
            if button.did_mouse_enter() {
                context.sound_library.sound_player.play(context.sound_library.hover.get());
            }
        }

        self.simple_texture.send();

        renderer.clear(&[ClearField::Color]);
        self.simple_texture_renderer.draw(projection_matrix, &mut self.simple_texture);
    }
}
