
pub mod graphics;

use glfw::{Action, Context, Key, PWindow};
use nalgebra_glm as glm;
use vertex_derive::{GlVertex, program_interface};
use crate::graphics::renderer::{Bindable, BlendFactor, Capability, ClearField, Renderer, buffer::{BufferUsage, VertexBuffer}, drawable::{DrawMode, Drawable}, fonts::Font, program::{Program, ShaderType}, texture::Texture, uniform::Uniform, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};

#[repr(C)]
#[derive(GlVertex)]
struct Vertex {
	position: glm::Vec2,
    texture_coord: glm::Vec2,
}

#[program_interface(
	vert = "../res/shaders/triangle.vert",
	frag = "../res/shaders/triangle.frag"
)]
struct TextureProgram {
	u_texture: i32,
    u_projection: glm::Mat4,
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

    let font_texture = Font::from_bytes(renderer, include_bytes!("../res/fonts/FreeMono.ttf")).unwrap();
    font_texture.bind_to_unit(0);

    renderer.enable(Capability::Blend);
    renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

    let mut vbo = VertexBuffer::<Vertex>::new(&renderer);
    vbo.set_data(&[
        Vertex {
            position: glm::Vec2::new(-0.5,  0.5),
            texture_coord: glm::Vec2::new(0.0, 0.0),
        },
        Vertex {
            position: glm::Vec2::new(-0.5, -0.5),
            texture_coord: glm::Vec2::new(0.0, 1.0),
        },
        Vertex {
            position: glm::Vec2::new( 0.5, -0.5),
            texture_coord: glm::Vec2::new(1.0, 1.0),
        },
        Vertex {
            position: glm::Vec2::new( 0.5,  0.5),
            texture_coord: glm::Vec2::new(1.0, 0.0),
        }
    ], BufferUsage::StaticDraw);

    let vao = VertexArrayObject::new(&[&vbo]);
    vao.bind();

    let program = TextureProgram::init(renderer).unwrap();
    program.bind();
    program.u_texture.set(&0);

    // let texture = Texture::from_image_bytes(
    //     renderer,
    //     include_bytes!("../res/textures/Frame-1.png"),
    //     graphics::renderer::texture::MagFiltering::Nearest,
    //     graphics::renderer::texture::MinFiltering::LinearMipmapLinear,
    //     graphics::renderer::texture::TextureWrap::ClampToEdge
    // ).unwrap();
    // texture.bind_to_unit(0);

    renderer.clear_color(&glm::Vec4::new(0.1, 0.2, 0.3, 0.0));

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

        program.u_projection.set(&get_projection_matrix(window));
        vao.draw(4, DrawMode::TriangleFan);

        window.swap_buffers();
    }
}
