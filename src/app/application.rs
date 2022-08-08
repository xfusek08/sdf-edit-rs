use bevy_ecs::{world::World, schedule::{Schedule, SystemStage, Stage}, system::{Res, Query, ResMut}};
use dolly::prelude::YawPitch;
use glam::Vec3;
use winit::{window::Window, event::{WindowEvent, ElementState, KeyboardInput, VirtualKeyCode}};

use super::{
    renderer::Renderer, clock::Tick,
    camera::Camera,
    components::{Mesh, Texture},
    model::{PENTAGON_VERTICES, PENTAGON_INDICES}
};

#[derive(Default)]
pub struct ApplicationConfig;

pub struct Application {
    world: World,
    update_scheduler: Schedule,
    render_scheduler: Schedule,
}

// static
impl Application {
    
    #[profiler::function]
    pub async fn new(window: &Window, _config: ApplicationConfig) -> Self {
        
        let mut world = World::default();
        
        // add camera to scene
        world.spawn().insert(Camera::new().orbit(Vec3::ZERO, 10.0));
        
        // add model to scene
        world.spawn()
            .insert(Mesh { vertices: PENTAGON_VERTICES, indices: PENTAGON_INDICES })
            .insert(Texture {
                texture: image::load_from_memory(
                    include_bytes!("../../resources/textures/happy-tree.png")
                ).expect("Failed fo load texture image.")
            });
            
        let mut update_scheduler = Schedule::default();
        update_scheduler
            .add_stage("update", SystemStage::parallel()
                .with_system(Self::update_camera),
            );
        
        let mut render_scheduler = Schedule::default();
        render_scheduler
            .add_stage("prepare", SystemStage::parallel()
                .with_system(Renderer::prepare_system)
            )
            .add_stage("render", SystemStage::single_threaded()
                .with_system(Renderer::render_system)
            );
            
        world.insert_resource(Renderer::new(window).await);
        
        return Self {
            render_scheduler,
            update_scheduler,
            world,
        };
    }
    
    #[profiler::function]
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.world.resource_mut::<Renderer>().resize(new_size);
    }
    
    #[profiler::function]
    pub fn input(&mut self, event: &WindowEvent) -> UpdateResult {
        if let WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } = event
        {
            let mut res = UpdateResult::Redraw;
            let mut camera = self.world.query::<&mut Camera>().get_single_mut(&mut self.world).unwrap();
            let controller = camera.rig.driver_mut::<YawPitch>();
            match keycode {
                VirtualKeyCode::Escape => return UpdateResult::Exit,
                VirtualKeyCode::W => controller.rotate_yaw_pitch(0.0, 10.0),
                VirtualKeyCode::A => controller.rotate_yaw_pitch(0.0, -10.0),
                VirtualKeyCode::S => controller.rotate_yaw_pitch(10.0, 0.0),
                VirtualKeyCode::D => controller.rotate_yaw_pitch(-10.0, 0.0),
                _ => res = UpdateResult::Wait,
            }
            return res;
        }
        UpdateResult::Wait
    }
    
    #[profiler::function]
    pub fn update(&mut self, tick: &Tick) {
        self.world.insert_resource(tick.clone());
        self.update_scheduler.run(&mut self.world);
    }
    
    #[profiler::function]
    pub fn render(&mut self) {
        self.render_scheduler.run(&mut self.world);
    }
    
    #[profiler::function]
    pub fn update_camera(
        tick: Res<Tick>,
        mut camera_query: Query<&mut Camera>
    ) {
        if let Ok(mut camera) = camera_query.get_single_mut() {
            camera.rig.update(tick.delta.as_secs_f32());
        }
    }

}

pub enum UpdateResult {
    Wait, Redraw, Exit
}