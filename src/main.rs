
pub mod graphics;

use glfw::{Action, Context, Key, PWindow};
use nalgebra_glm as glm;
use crate::graphics::renderer::{BlendFactor, Capability, ClearField, fonts::{Fonts}};

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

    let fonts = Fonts::new(renderer);
    let free_mono = fonts.new_font(renderer, include_bytes!("../res/fonts/FreeMono.ttf")).unwrap();
    let draw_buffer = free_mono.create_text_vbo(renderer, &fonts, "abcdefghijklmnopqrstuvwxyz", glm::vec2(0.0, 0.0), 0.15);

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
        fonts.draw_buffer(&draw_buffer, &proj_matrix);

        window.swap_buffers();
    }
}
