
use std::time;

use crate::{game::scene::Scene, graphics::renderer::{BlendFactor, Capability, ClearField, Renderer, buffer::{Buffer, Dynamic}, drawable::DrawMode, program::{Program, ShaderType}, texture::{MagFiltering, MinFiltering, Texture, TextureWrap}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}}};
use glfw::{Action, Glfw, GlfwReceiver, Key, WindowEvent};
use nalgebra_glm as glm;
use vertex_derive::{GlVertex, program_interface};

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

    fn get_texture_coords(&self) -> [glm::Vec2; 4] {
        let diffs = [
            glm::vec2(0.0, 0.5),
            glm::vec2(0.0, 0.0),
            glm::vec2(0.5, 0.0),
            glm::vec2(0.5, 0.5),
        ];

        let sum_diffs = |start: glm::Vec2| [
            start + diffs[0],
            start + diffs[1],
            start + diffs[2],
            start + diffs[3],
        ];

        match self.current_index {
            0 => sum_diffs(glm::vec2(0.0, 0.0)),
            1 | 4 => sum_diffs(glm::vec2(0.5, 0.0)),
            2 => sum_diffs(glm::vec2(0.0, 0.5)),
            3 => sum_diffs(glm::vec2(0.5, 0.5)),
            _ => unreachable!(),
        }
    }
}

#[program_interface(
	vert = "../../res/shaders/texture.vert",
	frag = "../../res/shaders/texture.frag"
)]
struct TextureProgram {
    u_projection: glm::Mat4,
    u_texture: i32,
}

#[repr(C)]
#[derive(GlVertex)]
struct TextureVertex {
    position: glm::Vec2,
    tex_coord: glm::Vec2,
}

pub struct Playing<'a> {
    position: glm::Vec2,
    vertical_speed: f32,
    atlas: Texture<'a>,
    vbo: Buffer<'a, TextureVertex, Dynamic>,
    vao: VertexArrayObject<'a>,
    program: TextureProgram<'a>,
    bird_fly_state: BirdFlyState,
    last_frame_time: std::time::Instant,
}

impl<'a> Playing<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        renderer.enable(Capability::Blend);
        renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        renderer.clear_color(&glm::Vec4::new(0.1, 0.2, 0.3, 0.0));

        let atlas = Texture::from_image_bytes(
            renderer,
            include_bytes!("../../res/textures/atlas.png"),
            MagFiltering::Linear,
            MinFiltering::LinearMipmapLinear,
            TextureWrap::ClampToBorder
        ).unwrap();

        let vbo = Buffer::<TextureVertex, Dynamic>::new(renderer, 4);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);
        let program = TextureProgram::init(renderer).unwrap();
        program.set_u_texture(&0);

        return Self {
            position: glm::vec2(-0.8, 0.0),
            vertical_speed: 0.5,
            atlas,
            vao,
            vbo,
            program,
            bird_fly_state: BirdFlyState::new(),
            last_frame_time: time::Instant::now(),
        };
    }
}

const VERTICAL_SPEED: f32 = 0.5;

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
        self.position += glm::vec2(VERTICAL_SPEED, self.vertical_speed) * delta;
    }

    fn generate_output(&mut self, renderer: &Renderer, projection_matrix: &glm::Mat4) {
        let bird_dimensions = glm::vec2(717.0, 610.0);
        let bird_logic_height = 0.15f32;
        let scale = bird_logic_height / bird_dimensions.y;
        let bird_logic_size = bird_dimensions * scale;
        let velocity = glm::vec2(0.5, self.vertical_speed);
    	let direction = glm::normalize(&velocity);
        let rotation_matrix = glm::mat2(
            direction.x, -direction.y,
            direction.y,  direction.x
        );

        let vertice_variants = [
            glm::vec2(-0.5, -0.5),
            glm::vec2(-0.5,  0.5),
            glm::vec2( 0.5,  0.5),
            glm::vec2( 0.5, -0.5),
        ];

        let uv_variants = self.bird_fly_state.get_texture_coords();

        let calc_vert_position = |idx: usize|
            self.position + rotation_matrix * bird_logic_size.component_mul(&vertice_variants[idx]);


        self.vbo.set_sub_data(&[
            TextureVertex {
                position: calc_vert_position(0),
                tex_coord: uv_variants[0]
            },
            TextureVertex {
                position: calc_vert_position(1),
                tex_coord: uv_variants[1]
            },
            TextureVertex {
                position: calc_vert_position(2),
                tex_coord: uv_variants[2]
            },
            TextureVertex {
                position: calc_vert_position(3),
                tex_coord: uv_variants[3]
            },
        ], 0);

        self.program.bind();
        self.program.set_u_projection(projection_matrix);

        self.atlas.bind_to_unit(0);

        renderer.clear(&[ClearField::Color]);
        self.vao.draw(4, DrawMode::TriangleFan);
    }
}