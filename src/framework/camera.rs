
use std::marker::PhantomData;

use glam::{Vec3, Mat4};

use dolly::{
    driver::RigDriver,
    transform::Transform as DollyTransform,
    rig::{CameraRig, RigUpdateParams},
    prelude::{YawPitch, Smooth, Handedness, Position, RightHanded},
};

use super::{
    math::Transform,
    updater::{
        UpdaterModule,
        UpdateContext,
        InputUpdateResult,
        UpdateResultAction,
        ResizeContext,
        AfterRenderContext
    }
};

// CameraProperties
// ----------------

pub struct CameraProperties {
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Default for CameraProperties {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            fov: 90.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

// Camera
// ------

pub struct Camera {
    pub rig: dolly::rig::CameraRig,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    
    pub fn new(properties: CameraProperties) -> Self {
        Self {
            rig: CameraRig::builder().build(),
            aspect_ratio: properties.aspect_ratio,
            fov: properties.fov,
            near: properties.near,
            far: properties.far,
        }
    }
    
    pub fn orbit(mut self, center: Vec3, distance: f32) -> Self {
        self.rig = CameraRig::builder()
            .with(YawPitch::new())
            .with(Smooth::new_rotation(0.8))
            // .with(Smooth::new_rotation(0.3).predictive(true))
            .with(SmoothZoomArm::new((Vec3::Z * distance) - center, 0.8))
            .build();
        self
    }
    
}

impl Camera {
    
    #[profiler::function]
    pub fn view_matrix(&self) -> Mat4 {
        glam::Mat4::from_rotation_translation(
            self.rig.final_transform.rotation,
            self.rig.final_transform.position
        ).inverse()
    }
    
    #[profiler::function]
    pub fn projection_matrix(&self) -> Mat4 {
        glam::Mat4::perspective_rh(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.near,
            self.far
        )
    }
    
    #[profiler::function]
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
    
    pub fn transform(&self) -> Transform {
        Transform {
            position: self.rig.final_transform.position,
            rotation: self.rig.final_transform.rotation,
            ..Default::default()
        }
    }
}

// SmoothZoomArm
// -------------

/// This is a custom dolly rig driver that behaves just like Arm but smooths a offset vale
/// Implementation based on example: https://github.com/h3r2tic/dolly/blob/main/examples/nested_driver.rs
/// Offsets the camera along a vector, in the coordinate space of the parent.
#[derive(Debug)]
pub struct SmoothZoomArm<H: Handedness> {
    direction: Vec3,
    smooth_rig: CameraRig<H>,
}

impl<H: Handedness> SmoothZoomArm<H> {
    pub fn new(offset: Vec3, smoothness: f32) -> Self {
        let magnitude = offset.length();
        Self {
            direction: offset.normalize(),
            smooth_rig: CameraRig::builder()
                .with(Position::new(Vec3::new(magnitude, 0.0, 0.0)))
                .with(Smooth::new_position(smoothness))
                .build(),
        }
    }
    
    pub fn scale_distance(&mut self, scale: f32) {
        let p = self.smooth_rig.driver_mut::<Position>();
        p.position.x = p.position.x * scale;
    }
}

impl<H: Handedness> RigDriver<H> for SmoothZoomArm<H> {
    fn update(&mut self, params: RigUpdateParams<H>) -> DollyTransform<H> {
        let t = self.smooth_rig.update(params.delta_time_seconds);
        DollyTransform {
            rotation: params.parent.rotation,
            position: params.parent.position + params.parent.rotation * (t.position.x * self.direction),
            phantom: PhantomData,
        }
    }
}

// SceneWithCamera Trait
// ---------------------

pub trait SceneWithCamera {
    fn get_camera(&self) -> &Camera;
    fn get_camera_mut(&mut self) -> &mut Camera;
}

// CameraUpdater
// -------------

#[derive(Default)]
pub struct CameraUpdater;

impl<S: SceneWithCamera> UpdaterModule<S> for CameraUpdater {
    
    #[profiler::function]
    fn input(&mut self, context: &mut UpdateContext<S>) -> InputUpdateResult {
        let camera = &mut context.scene.get_camera_mut();
        
        let (dx, dy) = context.input.mouse_diff();
        if (dx != 0.0 || dy != 0.0) && context.input.mouse_held(0) {
            camera
                .rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-dx * 0.7, -dy * 0.7);
        }
        let scroll = context.input.scroll_diff();
        if scroll != 0.0 {
            camera
                .rig
                .driver_mut::<SmoothZoomArm<RightHanded>>()
                .scale_distance(1.0 + scroll * -0.3);
        }
        
        InputUpdateResult::default() // do not prevent event propagation
    }
    
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext<S>) -> UpdateResultAction {
        let camera = &mut context.scene.get_camera_mut();
        
        let orig = camera.rig.final_transform;
        let new = camera.rig.update(context.tick.delta.as_secs_f32());
        if orig.position != new.position || orig.rotation != new.rotation {
            return UpdateResultAction::Redraw;
        }
        UpdateResultAction::None
    }
    
    #[profiler::function]
    fn resize(&mut self, context: &mut ResizeContext<S>) -> UpdateResultAction {
        let camera = &mut context.scene.get_camera_mut();
        camera.aspect_ratio = context.size.width as f32 / context.size.height as f32;
        UpdateResultAction::None
    }
    
    fn after_render(&mut self, _: &mut AfterRenderContext<S>) {}
    
}
