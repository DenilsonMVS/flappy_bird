
pub mod graphics;
pub mod sounds;
pub mod game;

use std::time;

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use nalgebra_glm::{self as glm, Mat4};
use crate::{game::{main_menu::MainMenu, scene::Scene}, graphics::renderer::{Renderer, fonts::{Fonts, TextRenderConfig}, positioning::{PositionMode, screen_pos_to_world_pos}}, sounds::{Sound, Sounds}};

fn get_window_size(window: &PWindow) -> glm::Vec2 {
    let (x, y) = window.get_size();
    return glm::vec2(x as f32, y as f32);
}

fn get_projection_matrix(window_size: glm::Vec2) -> glm::Mat4 {
    let ratio = window_size.x / window_size.y;

    let projection = if window_size.x > window_size.y {
        glm::ortho(-ratio, ratio, -1.0, 1.0, -1.0, 1.0)
    } else {
        let inv_ratio = window_size.y / window_size.x;
        glm::ortho(-1.0, 1.0, -inv_ratio, inv_ratio, -1.0, 1.0)
    };

    return projection;
}

fn get_mouse_pos(window: &PWindow, window_size: glm::Vec2, i_projection_matrix: &glm::Mat4) -> glm::Vec2 {
    let (x, y) = window.get_cursor_pos();
    return screen_pos_to_world_pos(glm::vec2(x as f32, y as f32), window_size, i_projection_matrix);
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

#[allow(dead_code)]
pub struct MainContext<'a> {
    glfw: &'a mut Glfw,
    window: &'a mut PWindow,
    events: &'a mut GlfwReceiver<(f64, WindowEvent)>,
    renderer: &'a Renderer,
    proj_matrix: Mat4,
    i_proj_matrix: Mat4,
    now: std::time::Instant,
    sound_library: &'a SoundLibrary,
    window_size: glm::Vec2,
    mouse_pos: glm::Vec2,
}

fn main() {

    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();
    
    let sound_library = SoundLibrary::new().unwrap();

    let fonts = Fonts::new(renderer);
    let deja_vu_sans = fonts.new_font(renderer, include_bytes!("../res/fonts/DejaVuSans.ttf")).unwrap();

    let draw_buffer = deja_vu_sans.create_text_vbo(renderer, &[
        TextRenderConfig::new(
            "abcdefghijklmnopqrstuvwxyz",
            glm::vec2(0.0, 0.0),
            0.08,
            glm::vec4(20, 10, 50, 255),
        ),
        TextRenderConfig::new(
            "0123456789",
            glm::vec2(-1.0, 1.0),
            0.1,
            glm::vec4(20, 100, 50, 255),
        ).with_mode(PositionMode::TopLeft),
        TextRenderConfig::new(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            glm::vec2(1.0, -1.0),
            0.08,
            glm::vec4(200, 10, 50, 255),
        ).with_mode(PositionMode::BottomRight),
    ]);

    let mut current_scene: Box<dyn Scene> = Box::new(MainMenu::new(renderer));

    while !window.should_close() {
        
        let window_size = get_window_size(window);
        unsafe { gl::Viewport(0, 0, window_size.x as i32, window_size.y as i32); }

        let proj_matrix = get_projection_matrix(window_size);
        let i_proj_matrix = glm::inverse(&proj_matrix);
        let mouse_pos = get_mouse_pos(window, window_size, &i_proj_matrix);

        let mut main_context = MainContext {
            glfw, window, events, renderer,
            proj_matrix,
            i_proj_matrix,
            now: time::Instant::now(),
            sound_library: &sound_library,
            window_size,
            mouse_pos,
        };

        current_scene.as_mut().handle_input(&mut main_context);
        current_scene.as_mut().game_logic(&mut main_context);
        current_scene.as_mut().generate_output(&mut main_context);

        fonts.draw_buffer(&draw_buffer, &proj_matrix);

        window.swap_buffers();
    }
}
