
pub mod graphics;
pub mod sounds;
pub mod game;

use std::time;

use glfw::{Context, PWindow};
use nalgebra_glm as glm;
use crate::{game::{playing::Playing, scene::Scene}};

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

// const HOVER: &'static [u8] = include_bytes!("../res/sounds/hover.wav");
// const JUMP: &'static [u8] = include_bytes!("../res/sounds/jump.wav");
// const START: &'static [u8] = include_bytes!("../res/sounds/start.wav");

fn main() {
    // let sounds = Sounds::new().unwrap();
    // let hover = Sound::new(HOVER).unwrap();
    // let jump = Sound::new(JUMP).unwrap();
    // let start = Sound::new(START).unwrap();

    // sounds.play(hover.get().amplify(10.0));
    // sounds.play(jump.get().speed(0.5));
    // sounds.play(start.get().delay(std::time::Duration::from_millis(500)));

    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();

    // let fonts = Fonts::new(renderer);
    // let free_mono = fonts.new_font(renderer, include_bytes!("../res/fonts/FreeMono.ttf")).unwrap();
    // let draw_buffer = free_mono.create_text_vbo(renderer, &[
    //     TextRenderConfig::new(
    //         "abcdefghijklmnopqrtuvwxyz",
    //         glm::vec2(0.0, 0.0),
    //         0.08
    //     ),
    //     TextRenderConfig::new(
    //         "0123456789",
    //         glm::vec2(-1.0, 1.0),
    //         0.1
    //     ).with_mode(PositionMode::TopLeft),
    //     TextRenderConfig::new(
    //         "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    //         glm::vec2(1.0, -1.0),
    //         0.08
    //     ).with_mode(PositionMode::BottomRight),
    // ]);

    let mut current_scene: Box<dyn Scene> = Box::new(Playing::new(renderer));

    while !window.should_close() {
        
        let now = time::Instant::now();
        current_scene.as_mut().handle_input(glfw, events, &now);
        current_scene.as_mut().game_logic(&now);

        let proj_matrix = get_projection_matrix(window);
        current_scene.as_mut().generate_output(renderer, &proj_matrix);

        // fonts.draw_buffer(&draw_buffer, &proj_matrix);

        window.swap_buffers();
    }
}
