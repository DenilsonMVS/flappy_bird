
pub mod graphics;
pub mod sounds;
pub mod game;

use std::time;

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use nalgebra_glm::{self as glm, Mat4};
use crate::{game::{playing::Playing, scene::Scene}, graphics::renderer::Renderer, sounds::{Sound, Sounds}};

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

pub struct SoundLibrary {
    pub sound_player: Sounds,
    pub hover: Sound,
    pub jump: Sound,
    pub start: Sound,
}

impl SoundLibrary {
    fn new() -> Option<SoundLibrary> {
        Some(Self {
            sound_player: Sounds::new().ok()?,
            hover: Sound::new(HOVER).ok()?,
            jump: Sound::new(JUMP).ok()?,
            start: Sound::new(START).ok()?,
        })
    }
}

pub struct MainContext<'a> {
    glfw: &'a mut Glfw,
    window: &'a mut PWindow,
    events: &'a mut GlfwReceiver<(f64, WindowEvent)>,
    renderer: &'a Renderer,
    proj_matrix: Mat4,
    now: std::time::Instant,
    sound_library: &'a SoundLibrary,
}

fn main() {

    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();
    
    let sound_library = SoundLibrary::new().unwrap();

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
        
        let proj_matrix = get_projection_matrix(window);

        let mut main_context = MainContext {
            glfw, window, events, renderer,
            proj_matrix: proj_matrix,
            now: time::Instant::now(),
            sound_library: &sound_library
        };

        current_scene.as_mut().handle_input(&mut main_context);
        current_scene.as_mut().game_logic(&mut main_context);
        current_scene.as_mut().generate_output(&mut main_context);

        // fonts.draw_buffer(&draw_buffer, &proj_matrix);

        window.swap_buffers();
    }
}
