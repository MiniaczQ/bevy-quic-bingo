pub fn root_element<R>(
    ui: &mut egui::Context,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<egui::InnerResponse<egui::InnerResponse<R>>> {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.centered_and_justified(|ui| ui.vertical_centered(add_contents))
    })
}
