use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use crate::error;

use super::{
    scene::{Scene, components::Deleted},
    rendering::{Renderer, render_modules::line_render_module::LinesRenderModule},
    updating::{Updater, UpdateResult},
    clock::Tick
};

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application {
    scene: Option<Scene>,
    updater: Updater,
    renderer: Renderer,
}

// static
impl Application {
    
    #[profiler::function]
    pub async fn new(window: &Window, _config: ApplicationConfig) -> Self {
        return Self {
            renderer: Renderer::new(window).await
                .with_module::<LinesRenderModule>(),
            updater: Updater::new(),
            scene: Some(Scene::new()),
        };
    }
    
    #[profiler::function]
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }
    
    #[profiler::function]
    pub fn input(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult {
        let scene = self.scene.take().unwrap();
        let (result, scene) = self.updater.input(scene, input, tick);
        self.scene = Some(scene);
        result
    }
    
    #[profiler::function]
    pub fn update(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult {
        let scene = self.scene.take().unwrap();
        let (result, scene) = self.updater.update(scene, input, tick);
        self.scene = Some(scene);
        result
    }
    
    #[profiler::function]
    pub fn render(&mut self) {
        self.renderer.prepare(self.scene.as_ref().unwrap());
        self.renderer.render();
        // TODO: this is meant to run in a separate thread alongside the render thread
        self.finalize();
    }
    
    /// Remove deleted entities from scene
    #[profiler::function]
    pub fn finalize(&mut self) {
        let scene = self.scene.as_mut().unwrap();
        let mut entities_to_delete = Vec::with_capacity(scene.world.len() as usize);
        for (entity, (Deleted(deleted),)) in scene.world.query::<(&Deleted,)>().iter() {
            if *deleted {
                entities_to_delete.push(entity);
            }
        }
        for entity in entities_to_delete {
            if let Err(_) = scene.world.despawn(entity) {
                error!("Failed to despawn entity {:?}", entity);
            }
        }
        
        self.renderer.finalize(self.scene.as_mut().unwrap());
    }
}
