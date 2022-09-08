use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use crate::error;

use super::{
    scene::{Scene, components::Deleted},
    rendering::{Renderer, render_modules::line_render_module::LinesRenderModule},
    updating::Updater,
    clock::Tick,
    // gui::Gui
};

pub enum UpdateResult {
    Wait, Redraw, Exit
}

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application {
    scene: Scene,
    updater: Updater,
    renderer: Renderer,
    // gui: Gui,
}

// static
impl Application {
    
    #[profiler::function]
    pub async fn new(window: &Window, _config: ApplicationConfig) -> Self {
        
        let renderer = Renderer::new(window).await
            .with_module::<LinesRenderModule>();
            
        // let gui = Gui::new(window, &renderer.context);
        
        return Self {
            renderer,
            // gui,
            updater: Updater::new(),
            scene: Scene::new(),
        };
    }
    
    #[profiler::function]
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }
    
    #[profiler::function]
    pub fn input(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult {
        self.updater.input(
            &mut self.scene,
            input,
            tick
        )
    }
    
    #[profiler::function]
    pub fn update(&mut self, input: &WinitInputHelper, tick: &Tick) -> UpdateResult {
        self.updater.update(
            &mut self.scene,
            input,
            tick
        )
    }
    
    #[profiler::function]
    pub fn render(&mut self) {
        self.renderer.prepare(&self.scene);
        
        self.renderer.render();
        
        // TODO: this is meant to run in a separate thread alongside the render thread
        self.finalize();
    }
    
    #[profiler::function]
    pub fn finalize(&mut self) {
        
        // fill buffer with entities to delete
        let scene = &self.scene;
        let mut entities_to_delete = Vec::with_capacity(scene.world.len() as usize);
        for (entity, (Deleted(deleted),)) in scene.world.query::<(&Deleted,)>().iter() {
            if *deleted {
                entities_to_delete.push(entity);
            }
        }
        
        // delete entities
        for entity in entities_to_delete {
            if let Err(_) = self.scene.world.despawn(entity) {
                error!("Failed to despawn entity {:?}", entity);
            }
        }
        
        // let renderer update state of components it is concerned with
        self.renderer.finalize(&mut self.scene);
    }
}
