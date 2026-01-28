use crate::{graphics::renderer::Renderer};
use glfw::{Glfw, GlfwReceiver, WindowEvent};
use nalgebra_glm as glm;

pub trait Scene {
    fn handle_input(&mut self, glfw: &mut Glfw, events: &mut GlfwReceiver<(f64, WindowEvent)>, now: &std::time::Instant);
    fn game_logic(&mut self, now: &std::time::Instant);
    fn generate_output(&mut self, renderer: &Renderer, projection_matrix: &glm::Mat4);
}

