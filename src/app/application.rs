use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use super::{
    scene::Scene,
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
        
        // // TODO: remove deleted entities from ecs
        // //  - prepare stage deleted allocated rendering resources for them
        // //  - Delete deleted entities in another parallel with rendering
        // -> delete it here <-
        
        self.renderer.render();
    }
}
