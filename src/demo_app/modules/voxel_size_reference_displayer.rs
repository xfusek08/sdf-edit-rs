use crate::{
    demo_app::scene::Scene,
    framework::updater::UpdaterModule,
};

use super::{
    tmp_evaluator_config::VoxelSizeOutlineComponent,
    cube::CubeOutlineComponent,
};

pub struct VoxelSizeReferenceDisplayer {
    pub visible: bool,
}

impl VoxelSizeReferenceDisplayer {
    pub fn show(&mut self, scene: &mut Scene) {
        self.visible = true;
        scene.world.spawn((
            VoxelSizeOutlineComponent,
            CubeOutlineComponent::new(1.5, 0.0, 0.0, scene.tmp_evaluator_config.min_voxel_size)
        ));
    }
    pub fn hide(&mut self, scene: &mut Scene) {
        self.visible = false;
        let to_delete: Vec<hecs::Entity> = scene.world.query::<&VoxelSizeOutlineComponent>()
            .iter()
            .map(|(e,_)| e)
            .collect();
                
        for e in to_delete {
            scene.world.despawn(e).unwrap();
        }
    }
}

impl UpdaterModule<Scene> for VoxelSizeReferenceDisplayer {
    fn input(&mut self, _: &mut crate::framework::updater::UpdateContext<Scene>) -> crate::framework::updater::InputUpdateResult {
        crate::framework::updater::InputUpdateResult::default()
    }

    fn update(&mut self, context: &mut crate::framework::updater::UpdateContext<Scene>) -> crate::framework::updater::UpdateResultAction {
        if context.scene.display_toggles.show_voxel_size_reference != self.visible {
            if context.scene.display_toggles.show_voxel_size_reference {
                self.show(&mut context.scene);
            } else {
                self.hide(&mut context.scene);
            }
        }
        crate::framework::updater::UpdateResultAction::None
    }

    fn resize(&mut self, _: &mut crate::framework::updater::ResizeContext<Scene>) -> crate::framework::updater::UpdateResultAction {
        crate::framework::updater::UpdateResultAction::None
    }

    fn after_render(&mut self, _: &mut crate::framework::updater::AfterRenderContext<Scene>) {}
}
