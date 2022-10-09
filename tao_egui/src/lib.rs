use std::{sync::Arc, time::Instant};

use egui_glow::{egui_winit::egui, EguiGlow};
use glutin::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    PossiblyCurrent, WindowedContext,
};

pub struct TaoEguiWindow {
    gl_window: WindowedContext<PossiblyCurrent>,
    gl: Arc<glow::Context>,
    egui_glow: EguiGlow,
}

impl TaoEguiWindow {
    pub fn new(event_loop: &EventLoop<()>, window_title: impl Into<String>) -> Self {
        let (gl_window, gl) = create_display(event_loop, window_title);
        let gl = std::sync::Arc::new(gl);
        let egui_glow = egui_glow::EguiGlow::new(event_loop, gl.clone());

        Self {
            gl_window,
            gl,
            egui_glow,
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Event<()>,
        do_ui: impl FnMut(&egui::Context),
    ) -> Option<ControlFlow> {
        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            Event::RedrawEventsCleared if cfg!(windows) => {
                return self.do_draw(do_ui);
            }
            Event::RedrawRequested(_) if !cfg!(windows) => {
                return self.do_draw(do_ui);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                self.gl_window.window().request_redraw();
            }
            Event::WindowEvent {
                window_id, event, ..
            } if self.gl_window.window().id() == *window_id => {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        // TODO: do we want to exit the application when egui is closed?
                        return Some(ControlFlow::Exit);
                    }
                    WindowEvent::Resized(physical_size) => {
                        self.gl_window.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.gl_window.resize(**new_inner_size);
                    }
                    _ => (),
                }
                self.egui_glow.on_event(event);
                self.gl_window.window().request_redraw();
            }
            Event::LoopDestroyed => {
                self.egui_glow.destroy();
            }

            _ => (),
        }

        None
    }

    fn do_draw(&mut self, do_ui: impl FnMut(&egui::Context)) -> Option<ControlFlow> {
        let repaint_after = self.egui_glow.run(self.gl_window.window(), do_ui);

        let control_flow = if repaint_after.is_zero() {
            self.gl_window.window().request_redraw();
            Some(ControlFlow::Poll)
        } else {
            Some(ControlFlow::WaitUntil(Instant::now() + repaint_after))
        };

        unsafe {
            use glow::HasContext as _;
            self.gl.clear_color(0f32, 0f32, 0f32, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        self.egui_glow.paint(self.gl_window.window());
        self.gl_window.swap_buffers().unwrap();

        control_flow
    }
}

// NOTE: adapted from tao/winit example code
fn create_display(
    event_loop: &EventLoop<()>,
    window_title: impl Into<String>,
) -> (
    glutin::WindowedContext<glutin::PossiblyCurrent>,
    glow::Context,
) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title(window_title);

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

    (gl_window, gl)
}
