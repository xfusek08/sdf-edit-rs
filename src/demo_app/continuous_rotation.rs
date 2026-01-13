/// Updater module that will rotate entities that has transform and geometry id in random direction
/// the random rotation will be attached as a component to the entity in the scene
use crate::framework::{math::Transform, updater::UpdaterModule};

use super::scene::Scene;

pub struct ContinuousRotation {
    quad: glam::Quat,
}

impl ContinuousRotation {
    pub fn from_speed_axis(speed: f32, axis: glam::Vec3) -> Self {
        let rotation = glam::Quat::from_axis_angle(axis, speed);
        Self { quad: rotation }
    }

    pub fn random() -> Self {
        let speed = rand::random::<f32>() * 0.1;
        let axis = glam::Vec3::new(
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        )
        .normalize();
        Self::from_speed_axis(speed, axis)
    }

    /// will rotate the transform by the speed and direction
    fn increment(&self, transform: &Transform) -> Transform {
        let rotation = transform.rotation * self.quad;
        Transform {
            rotation,
            ..*transform
        }
    }
}

pub struct ContinuousRotator;

impl UpdaterModule<Scene> for ContinuousRotator {
    fn input(
        &mut self,
        _: &mut crate::framework::updater::UpdateContext<Scene>,
    ) -> crate::framework::updater::InputUpdateResult {
        crate::framework::updater::InputUpdateResult::default()
    }

    fn update(
        &mut self,
        context: &mut crate::framework::updater::UpdateContext<Scene>,
    ) -> crate::framework::updater::UpdateResultAction {
        let mut cnt = 0;
        for (_, (transform, continuous_rotation)) in context
            .scene
            .world
            .query_mut::<(&mut Transform, &ContinuousRotation)>()
        {
            *transform = continuous_rotation.increment(&transform);
            cnt += 1;
        }
        if cnt > 0 {
            crate::framework::updater::UpdateResultAction::Redraw
        } else {
            crate::framework::updater::UpdateResultAction::None
        }
    }

    fn resize(
        &mut self,
        _: &mut crate::framework::updater::ResizeContext<Scene>,
    ) -> crate::framework::updater::UpdateResultAction {
        crate::framework::updater::UpdateResultAction::None
    }

    fn after_render(&mut self, _: &mut crate::framework::updater::AfterRenderContext<Scene>) {}
}
