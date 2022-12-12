
pub trait GuiModule<Scene> {
    fn gui(&mut self, scene: &mut Scene, ui: &mut egui::Ui);
}
