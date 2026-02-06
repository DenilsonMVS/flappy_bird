
pub mod graphics;
pub mod sounds;
pub mod game;

use std::time;

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use macros::atlas_bundle;
use nalgebra_glm::{self as glm, Mat4};
use crate::{game::{main_menu::MainMenu, playing::Playing, scene::Scene}, graphics::renderer::{Renderer, atlas::{FrameInfo, TypedAtlas}, fonts::{Font, Fonts}, positioning::screen_pos_to_world_pos, simple_texture::{SimpleTexture, SimpleTextureRenderer}, texture::{MagFiltering, MinFiltering, TextureWrap}}, sounds::{Sound, Sounds}};
use anyhow::Result;

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
const DEATH: &'static [u8] = include_bytes!("../res/sounds/death.wav");

pub struct SoundLibrary {
    pub sound_player: Sounds,
    pub hover: Sound,
    pub jump: Sound,
    pub start: Sound,
    pub death: Sound,
}

impl SoundLibrary {
    fn new() -> Result<SoundLibrary> {
        Ok(Self {
            sound_player: Sounds::new()?,
            hover: Sound::new_static(HOVER)?,
            jump: Sound::new_static(JUMP)?,
            start: Sound::new_static(START)?,
            death: Sound::new_static(DEATH)?,
        })
    }
}

pub struct FontLibrary<'a> {
    pub fonts: Fonts<'a>,
    pub deja_vu_sans: Font<'a>,
}

impl<'a> FontLibrary<'a> {
    fn new(renderer: &'a Renderer) -> Result<Self> {
        let fonts = Fonts::new(renderer);
        let deja_vu_sans = fonts.new_font(renderer, include_bytes!("../res/fonts/DejaVuSans.ttf"))?;
        return Ok(Self { fonts, deja_vu_sans });
    }
}

pub struct TextureLibrary<'a> {
    simple_texture_renderer: SimpleTextureRenderer<'a>,
    simple_texture: SimpleTexture<'a, Atlas>,
}

impl<'a> TextureLibrary<'a> {
    fn new(renderer: &'a Renderer) -> Result<Self> {
        Ok(Self {
            simple_texture_renderer: SimpleTextureRenderer::new(renderer),
            simple_texture: SimpleTexture::new(
                renderer,
                include_bytes!("../res/textures/atlas/texture.png"),
                MagFiltering::Nearest, MinFiltering::Nearest, TextureWrap::ClampToBorder,
                include_bytes!("../res/textures/atlas/texture.json")
            )?
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NextScene {
    MainMenu,
    Playing
}

#[atlas_bundle("res/textures/atlas/texture.json")]
pub struct Atlas;

#[allow(dead_code)]
pub struct MainContext<'a, 'b> {
    // 'b current frame lifetime
    pub glfw: &'b mut Glfw,
    pub window: &'b mut PWindow,
    pub events: &'b mut GlfwReceiver<(f64, WindowEvent)>,
    pub next_scene: &'b mut Option<NextScene>,
    pub texture_library: &'b mut TextureLibrary<'a>,
    pub font_library: &'b mut FontLibrary<'a>,

    // 'a long lifetime
    pub renderer: &'a Renderer,
    pub sound_library: &'a SoundLibrary,
    
    pub proj_matrix: Mat4,
    pub i_proj_matrix: Mat4,
    pub now: std::time::Instant,
    pub window_size: glm::Vec2,
    pub mouse_pos: glm::Vec2,
}

fn main() {

    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Flappy Bird").unwrap();
    let (glfw, window, events, renderer) = setup.get();
    
    let sound_library = SoundLibrary::new().unwrap();
    let mut font_library= FontLibrary::new(renderer).unwrap();
    let mut texture_library = TextureLibrary::new(renderer).unwrap();

    let mut current_scene: Box<dyn Scene> = Box::new(MainMenu::new(renderer, &font_library, &texture_library));
    let mut next_scene = None;


    while !window.should_close() {

        if let Some(scene) = next_scene {
            match scene {
                NextScene::MainMenu => current_scene = Box::new(MainMenu::new(renderer, &font_library, &texture_library)),
                NextScene::Playing => current_scene = Box::new(Playing::new()),
            };
            next_scene = None;
        }

        let window_size = get_window_size(window);
        unsafe { gl::Viewport(0, 0, window_size.x as i32, window_size.y as i32); }

        let proj_matrix = get_projection_matrix(window_size);
        let i_proj_matrix = glm::inverse(&proj_matrix);
        let mouse_pos = get_mouse_pos(window, window_size, &i_proj_matrix);

        let mut main_context = MainContext {
            glfw, window, events, renderer,
            next_scene: &mut next_scene,
            proj_matrix,
            i_proj_matrix,
            now: time::Instant::now(),
            sound_library: &sound_library,
            window_size,
            mouse_pos,
            font_library: &mut font_library,
            texture_library: &mut texture_library,
        };

        current_scene.as_mut().handle_input(&mut main_context);
        current_scene.as_mut().game_logic(&mut main_context);
        current_scene.as_mut().generate_output(&mut main_context);

        window.swap_buffers();
    }
}
