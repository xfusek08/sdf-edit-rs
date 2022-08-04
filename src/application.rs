use winit::{window::Window, event::{WindowEvent, ElementState, KeyboardInput}};

use crate::{renderer::Renderer, error};

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application {
    renderer: Renderer,
}

// static
impl Application {
    
    #[profiler::function]
    pub async fn new(window: &Window, _config: ApplicationConfig) -> Self {
        return Self {
            renderer: Renderer::new(window).await,
        };
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> UpdateResult {
        if let WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(code),
                    ..
                },
                ..
            } = event {
                return UpdateResult::Exit;
        }
        UpdateResult::Wait
    }

    pub fn update(&mut self) -> UpdateResult {
        UpdateResult::Wait
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render()
    }

}

pub enum UpdateResult {
    Wait, Redraw, Exit
}