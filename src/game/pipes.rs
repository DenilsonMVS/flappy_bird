use rand::{Rng, rngs::ThreadRng};
use nalgebra_glm as glm;
use crate::{Atlas, AtlasFrame, game::defs::{MAX_PIPE_CENTER_DIST, OFFSET_PIPE_DEL, PIPE_AMOUNT, PIPE_SPACE, PIPE_START_POSITION, PIPE_WIDTH, REPEATED_PIPES, SPACE_BETWEEN_PIPES}, graphics::renderer::{positioning::{BaseDimensions, PositionMode, SimpleTransform, scale_dimension}, simple_texture::SimpleTexture}};


#[derive(Debug, Clone, Copy)]
pub struct Pipe {
    x_position: f32,
    opening_y_position: f32,
}

impl Pipe {
    pub fn get_x_position(&self) -> f32 {
        self.x_position
    }

    pub fn get_opening_y_position(&self) -> f32 {
        self.opening_y_position
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pipes {
    pipes: [Pipe; PIPE_AMOUNT],
    smaller_pipe: usize,
}

impl Pipes {
    pub fn new(rng: &mut ThreadRng) -> Self {
        let pipes: [Pipe; PIPE_AMOUNT] = std::array::from_fn(|idx| {
            Pipe {
                x_position: PIPE_START_POSITION + (idx as f32) * SPACE_BETWEEN_PIPES,
                opening_y_position: rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            }
        });

        return Self {
            pipes,
            smaller_pipe: 0
        };
    }

    pub fn send(&self, x_pos: f32, simple_texture: &mut SimpleTexture<Atlas>) {
        let pipe_entrance_uv = simple_texture.get_frame_info(AtlasFrame::TopCano);
        let pipe_entrance_original_dimensions = pipe_entrance_uv.get_original_dimensions();
        let pipe_uv = simple_texture.get_frame_info(AtlasFrame::Cano);
        let pipe_original_dimensions = pipe_uv.get_original_dimensions();
        let pipe_entrance_height = scale_dimension(pipe_entrance_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));
        let pipe_height = scale_dimension(pipe_original_dimensions, BaseDimensions::Width(PIPE_WIDTH));

        for &pipe in self.pipes.iter() {
            let position = pipe.x_position - x_pos;

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

    pub fn get_pipes(&self) -> &[Pipe] {
        &self.pipes
    }

    pub fn game_logic(&mut self, x_pos: f32, rng: &mut ThreadRng) {
        if x_pos - self.pipes[self.smaller_pipe].x_position > OFFSET_PIPE_DEL {
            let prev_pipe = &self.pipes[self.smaller_pipe.wrapping_sub(1) % PIPE_AMOUNT];
            self.pipes[self.smaller_pipe] = Pipe {
                x_position: prev_pipe.x_position + SPACE_BETWEEN_PIPES,
                opening_y_position: rng.random_range(-MAX_PIPE_CENTER_DIST..MAX_PIPE_CENTER_DIST),
            };
            self.smaller_pipe += 1;
            self.smaller_pipe %= PIPE_AMOUNT;
        }   
    }
}