
pub mod graphics;

use glfw::{Action, Context, Key};
use nalgebra_glm as glm;
use vertex_derive::GlVertex;
use crate::graphics::renderer::{Bindable, ClearField, buffer::{BufferUsage, IndexBuffer, VertexBuffer}, drawable::{DrawMode, Drawable}, program::{Program, ShaderType}, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};

#[repr(C)]
#[derive(GlVertex)]
struct Vertex {
	position: glm::Vec2,
	
	#[vertex(normalized)]
	color: glm::U8Vec4,
}

fn main() {
    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();

    let mut vbo = VertexBuffer::<Vertex>::new(&renderer);
    vbo.set_data(&[
            Vertex {
                position: glm::Vec2::new( -0.5,  0.5),
                color: glm::U8Vec4::new(255, 200, 255, 255)
            },
            Vertex {
                position: glm::Vec2::new(-0.5, -0.5),
                color: glm::U8Vec4::new(255, 255, 255, 255)
            },
            Vertex {
                position: glm::Vec2::new( 0.5, -0.5),
                color: glm::U8Vec4::new(20, 255, 235, 255)
            },
            Vertex {
                position: glm::Vec2::new( 0.5,  0.5),
                color: glm::U8Vec4::new(20, 255, 25, 255)
            }
        ],
        BufferUsage::StaticDraw
    );

    let vao = VertexArrayObject::new(&[&vbo]);
    vao.bind();

    let program = Program::new(&renderer, &[
        (include_str!("../res/shaders/triangle.vert"), ShaderType::Vertex),
        (include_str!("../res/shaders/triangle.frag"), ShaderType::Fragment)
    ]).unwrap();

    program.bind();

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
        vao.draw(4, DrawMode::TriangleFan);

        window.swap_buffers();
    }
}
