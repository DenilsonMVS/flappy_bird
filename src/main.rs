
pub mod graphics;

use glfw::{Action, Context, Key};
use nalgebra_glm as glm;

use crate::graphics::renderer::ClearField;

fn main() {
    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();

    renderer.clear_color(&glm::Vec4::new(0.1, 0.2, 0.3, 0.0));

    while !window.should_close() {
        window.swap_buffers();

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
    }
}
