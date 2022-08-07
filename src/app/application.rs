use winit::{window::Window, event::{WindowEvent, ElementState, KeyboardInput}};

use crate::info;

use super::{scene::Scene, renderer::Renderer, model::{Model, PENTAGON_VERTICES, PENTAGON_INDICES}};

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application {
    scene: Scene,
    renderer: Renderer,
}

// static
impl Application {
    
    #[profiler::function]
    pub async fn new(window: &Window, _config: ApplicationConfig) -> Self {
        return Self {
            renderer: Renderer::new(window).await,
            scene: Scene { models: vec![
                Model {
                    vertices: PENTAGON_VERTICES,
                    indices: PENTAGON_INDICES,
                    texture: image::load_from_memory(include_bytes!("../../resources/textures/happy-tree.png")).expect("Failed fo load texture image.") }
            ] }
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
        self.renderer.prepare(&self.scene);
        self.renderer.render()
    }

}

pub enum UpdateResult {
    Wait, Redraw, Exit
}