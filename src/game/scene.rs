
use crate::MainContext;

pub trait Scene {
    fn handle_input(&mut self, context: &mut MainContext);
    fn game_logic(&mut self, context: &mut MainContext);
    fn generate_output(&mut self, context: &mut MainContext);
}

