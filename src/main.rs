
pub mod graphics;
pub mod sounds;

use glfw::{Action, Context, Key, PWindow};
use nalgebra_glm as glm;
use rodio::Source;
use crate::{graphics::renderer::{BlendFactor, Capability, ClearField, fonts::{Fonts, PositionMode, TextRenderConfig}}, sounds::{Sound, Sounds}};

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

const HOVER: &'static [u8] = include_bytes!("../res/sounds/hover.wav");
const JUMP: &'static [u8] = include_bytes!("../res/sounds/jump.wav");
const START: &'static [u8] = include_bytes!("../res/sounds/start.wav");

fn main() {
    let sounds = Sounds::new().unwrap();
    let hover = Sound::new(HOVER).unwrap();
    let jump = Sound::new(JUMP).unwrap();
    let start = Sound::new(START).unwrap();

    sounds.play(hover.get().amplify(10.0));
    sounds.play(jump.get().speed(0.5));
    sounds.play(start.get().delay(std::time::Duration::from_millis(500)));

    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();

    renderer.enable(Capability::Blend);
    renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    renderer.clear_color(&glm::Vec4::new(0.1, 0.2, 0.3, 0.0));

    let fonts = Fonts::new(renderer);
    let free_mono = fonts.new_font(renderer, include_bytes!("../res/fonts/FreeMono.ttf")).unwrap();
    let draw_buffer = free_mono.create_text_vbo(renderer, &[
        TextRenderConfig::new(
            "abcdefghijklmnopqrtuvwxyz",
            glm::vec2(0.0, 0.0),
            0.08
        ),
        TextRenderConfig::new(
            "0123456789",
            glm::vec2(-1.0, 1.0),
            0.1
        ).with_mode(PositionMode::TopLeft),
        TextRenderConfig::new(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            glm::vec2(1.0, -1.0),
            0.08
        ).with_mode(PositionMode::BottomRight),
    ]);

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
