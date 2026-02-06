use std::time;

use rand::{Rng, rngs::ThreadRng};
use nalgebra_glm as glm;
use crate::{Atlas, AtlasFrame, FontLibrary, MainContext, NextScene, TextureLibrary, game::{button::Button, defs::{BIRD_START_POSITION, HORIZONTAL_SPEED, MAX_PIPE_CENTER_DIST, OFFSET_PIPE_DEL, PIPE_AMOUNT, PIPE_SPACE, PIPE_START_POSITION, PIPE_WIDTH, REPEATED_PIPES, SCENE_HEIGHT, SPACE_BETWEEN_PIPES}, scene::Scene}, graphics::renderer::{BlendFactor, Capability, ClearField, Renderer, fonts::FontVbo, positioning::{BaseDimensions, PositionMode, SimpleTransform, scale_dimension}, simple_texture::SimpleTexture}};

const PLAY_BUTTON_CENTER: glm::Vec2 = glm::Vec2::new(0.0, -0.2);
const QUIT_BUTTON_CENTER: glm::Vec2 = glm::Vec2::new(0.0, -0.5);

#[derive(Debug, Clone, Copy)]
struct Pipe {
    x_position: f32,
    opening_y_position: f32,
}

pub struct MainMenu<'a> {
    x_offset: f32,
    pipes: [Pipe; PIPE_AMOUNT],
    smaller_pipe: usize,
    rng: ThreadRng,
    last_frame_time: std::time::Instant,
    buttons: [Button; 2],
    font_buf: FontVbo<'a>,
}

impl<'a> MainMenu<'a> {
    pub fn new(renderer: &'a Renderer, font_library: &FontLibrary, texture_library: &TextureLibrary) -> Self {
        renderer.enable(Capability::Blend);
        renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        renderer.clear_color(&glm::vec4(0.1, 0.2, 0.3, 0.0)); 
        
        let mut rng = rand::rng();
        let pipes: [Pipe; PIPE_AMOUNT] = std::array::from_fn(|idx| {
            Pipe {
                x_position: PIPE_START_POSITION + (idx as f32) * SPACE_BETWEEN_PIPES,
                opening_y_position: rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            }
        });

        let buttons = [
            Button::new(
                PLAY_BUTTON_CENTER,
                "Play",
                glm::vec4(30u8, 15u8, 80u8, 255u8),
                &texture_library.simple_texture,
            ),
            Button::new(
                QUIT_BUTTON_CENTER,
                "Quit",
                glm::vec4(30u8, 15u8, 80u8, 255u8),
                &texture_library.simple_texture,
            ),
        ];

        let text_render_config: [_; 2] = std::array::from_fn(|i|
            buttons[i].get_text_render_config());

        let font_buf = font_library.deja_vu_sans.create_text_vbo(renderer, &text_render_config);

        return Self {
            buttons,
            x_offset: BIRD_START_POSITION,
            pipes,
            smaller_pipe: 0,
            rng,
            last_frame_time: time::Instant::now(),
            font_buf,
        };
    }

    fn send_scene(&mut self, simple_texture: &mut SimpleTexture<Atlas>) {
        let scene_uv = simple_texture.get_frame_info(AtlasFrame::Scene);
        let scene_original_dimensions = scene_uv.get_original_dimensions();
        let scene_width = scale_dimension(scene_original_dimensions, BaseDimensions::Height(SCENE_HEIGHT));
        let scene_start_position = (-self.x_offset * 0.5).rem_euclid(scene_width) - scene_width * 3.0;

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
            let position = pipe.x_position - self.x_offset;

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
}

impl<'a> Scene for MainMenu<'a> {
    fn handle_input(&mut self, context: &mut MainContext) {
        for button in self.buttons.iter_mut() {
            button.update(context.mouse_pos);
        }

        context.glfw.poll_events();

        for (_, event) in glfw::flush_messages(context.events) {
            match event {
                glfw::WindowEvent::MouseButton(button, action, _) => {
                    if button == glfw::MouseButtonLeft && action == glfw::Action::Release {
                        if self.buttons[0].is_mouse_inside() {
                            *context.next_scene = Some(NextScene::Playing);
                            context.sound_library.sound_player.play(context.sound_library.start.get());
                        } else if self.buttons[1].is_mouse_inside() {
                            context.window.set_should_close(true);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    fn game_logic(&mut self, context: &mut MainContext) {
        if self.x_offset - self.pipes[self.smaller_pipe].x_position > OFFSET_PIPE_DEL {
            let prev_pipe = &self.pipes[self.smaller_pipe.wrapping_sub(1) % PIPE_AMOUNT];
            self.pipes[self.smaller_pipe] = Pipe {
                x_position: prev_pipe.x_position + SPACE_BETWEEN_PIPES,
                opening_y_position: self.rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            };
            self.smaller_pipe += 1;
            self.smaller_pipe %= PIPE_AMOUNT;
        }

        let now = context.now;
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        self.x_offset += HORIZONTAL_SPEED * delta;
    }

    fn generate_output(&mut self, context: &mut MainContext) {
        let renderer = context.renderer;
        let projection_matrix = &context.proj_matrix;
        let simple_texture = &mut context.texture_library.simple_texture;

        self.send_scene(simple_texture);
        self.send_pipes(simple_texture);

        for button in self.buttons.iter_mut() {
            button.submit_data(simple_texture);
            if button.did_mouse_enter() {
                context.sound_library.sound_player.play(context.sound_library.hover.get());
            }
        }

        simple_texture.send();

        renderer.clear(&[ClearField::Color]);
        context.texture_library.simple_texture_renderer.draw(projection_matrix, simple_texture);
        context.font_library.fonts.draw_buffer(&context.font_library.deja_vu_sans, &self.font_buf, &context.proj_matrix);
    }
}
