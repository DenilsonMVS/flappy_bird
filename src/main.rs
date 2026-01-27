
pub mod graphics;

use glfw::{Action, Context, Key, PWindow};
use nalgebra_glm as glm;
use vertex_derive::{program_interface};
use crate::graphics::renderer::{Bindable, BlendFactor, Capability, ClearField, Renderer, drawable::{DrawMode, Drawable}, fonts::{Font, GLYPH_SIZE, GLYPYH_MARGIN, PX_RANGE}, program::{Program, ShaderType}, uniform::Uniform, vertex_array_object::VertexArrayObject};

#[program_interface(
	vert = "../res/shaders/font.vert",
	frag = "../res/shaders/font.frag"
)]
struct FontProgram {
    u_texture: i32,
    u_projection: glm::Mat4,
    u_px_range: f32,
    u_glyph_size: u32,
    u_glyph_margin: u32,
}

fn get_projection_matrix(window: &PWindow) -> glm::Mat4 {
    let (width, height) = window.get_size();
    unsafe { gl::Viewport(0, 0, width, height); }

    let ratio = width as f32 / height as f32;

    let projection = if width > height {
        glm::ortho(-ratio, ratio, -1.0, 1.0, -1.0, 1.0)
    } else {
        let inv_ratio = height as f32 / width as f32;
        glm::ortho(-1.0, 1.0, -inv_ratio, inv_ratio, -1.0, 1.0)
    };

    return projection;
}

fn main() {
    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();

    renderer.enable(Capability::Blend);
    renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    renderer.clear_color(&glm::Vec4::new(0.1, 0.2, 0.3, 0.0));

    let font_texture = Font::from_bytes(renderer, include_bytes!("../res/fonts/FreeMono.ttf")).unwrap();
    font_texture.bind_to_unit(0);

    let (vbo, num_glyphs) = font_texture.create_text_vbo(renderer, "abcdefghijklmnopqrstuvwxyz", glm::vec2(0.0, 0.0), 0.15);
    let base_vbo = font_texture.get_vbo();
    let vao = VertexArrayObject::new(&[base_vbo, &vbo]);

    let font_program = FontProgram::init(renderer).unwrap();
    font_program.bind();
    font_program.u_texture.set(&0);
    font_program.u_px_range.set(&PX_RANGE);
    font_program.u_glyph_size.set(&(GLYPH_SIZE as u32));
    font_program.u_glyph_margin.set(&(GLYPYH_MARGIN as u32));

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                _ => {},
            }
        }

        renderer.clear(&[ClearField::Color]);

        let proj_matrix = get_projection_matrix(window);
        font_program.u_projection.set(&proj_matrix);

        vao.draw_instanced(4, num_glyphs as i32, DrawMode::TriangleFan);

        window.swap_buffers();
    }
}
