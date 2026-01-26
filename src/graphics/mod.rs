
pub mod renderer;

use std::{ffi::CStr, sync::atomic::{AtomicBool, Ordering}};

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent, fail_on_errors};
use nalgebra_glm as glm;


extern "system" fn gl_debug_output(
	source: gl::types::GLenum,
    gl_type: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void,
) {
	let source_str = match source {
		gl::DEBUG_SOURCE_API             => "API",
		gl::DEBUG_SOURCE_WINDOW_SYSTEM   => "Window System",
		gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
		gl::DEBUG_SOURCE_THIRD_PARTY     => "Third Party",
		gl::DEBUG_SOURCE_APPLICATION     => "Application",
		gl::DEBUG_SOURCE_OTHER           => "Other",
		_                                => "Unknown",
	};

	let type_str = match gl_type {
		gl::DEBUG_TYPE_ERROR               => "Error",
		gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated",
		gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR  => "Undefined Behavior",
		gl::DEBUG_TYPE_PORTABILITY         => "Portability",
		gl::DEBUG_TYPE_PERFORMANCE         => "Performance",
		gl::DEBUG_TYPE_MARKER              => "Marker",
		gl::DEBUG_TYPE_PUSH_GROUP          => "Push Group",
		gl::DEBUG_TYPE_POP_GROUP           => "Pop Group",
		gl::DEBUG_TYPE_OTHER               => "Other",
		_                                  => "Unknown",
	};

	let severity_str = match severity {
		gl::DEBUG_SEVERITY_HIGH         => "\x1b[31mHigh\x1b[0m",   // Vermelho
		gl::DEBUG_SEVERITY_MEDIUM       => "\x1b[33mMedium\x1b[0m", // Amarelo
		gl::DEBUG_SEVERITY_LOW          => "\x1b[32mLow\x1b[0m",    // Verde
		gl::DEBUG_SEVERITY_NOTIFICATION => "\x1b[34mNote\x1b[0m",   // Azul
		_                               => "Unknown",
	};

	let message_str = unsafe {
		CStr::from_ptr(message).to_str().unwrap_or("Failed to parse message")
	};

	eprintln!(
		"[GL Debug] [{}] Type: {} | Source: {} | ID: {}\nMessage: {}\n",
		severity_str, type_str, source_str, id, message_str
	);
}

pub struct Graphics {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    renderer: renderer::Renderer,
}

static GRAPHICS_INITIALIZED: AtomicBool = AtomicBool::new(false);

impl Graphics {
    pub fn new(window_dimensions: glm::U32Vec2, window_title: &str) -> Option<Self> {
        GRAPHICS_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).ok()?;
        
        let mut glfw = glfw::init(fail_on_errors!()).ok()?;

        let (mut window, events) = glfw.create_window(
            window_dimensions.x, window_dimensions.y, window_title,
            glfw::WindowMode::Windowed)?;

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

        window.make_current();
        window.set_key_polling(true);

        gl::load_with(|s| 
            window.get_proc_address(s).map_or(std::ptr::null(), |p| p as *const _)
        );

        if !gl::Clear::is_loaded() {
            return None;
        }

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

            gl::DebugMessageCallback(
                Some(gl_debug_output),
                std::ptr::null()
            );
        }

        return Some(Self {
            glfw,
            window,
            events,
            renderer: renderer::Renderer::new()
        });
    }

    pub fn get(&mut self) -> (&mut Glfw, &mut PWindow, &mut GlfwReceiver<(f64, WindowEvent)>, &mut renderer::Renderer) {
        return (&mut self.glfw, &mut self.window, &mut self.events, &mut self.renderer);
    }
}