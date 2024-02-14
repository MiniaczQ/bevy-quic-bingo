use std::collections::HashMap;

use bevy::ecs::system::Resource;

#[derive(Resource)]
pub struct PromptLayoutCache {
    downscale_factor: f32,
    base_font: egui::FontId,
    data: HashMap<String, egui::FontId>,
}

impl Default for PromptLayoutCache {
    fn default() -> Self {
        Self {
            downscale_factor: 0.9,
            base_font: Default::default(),
            data: Default::default(),
        }
    }
}

impl PromptLayoutCache {
    fn font(&mut self, painter: &egui::Painter, text: &str, rect: egui::Rect) -> egui::FontId {
        if let Some(size) = self.data.get(text) {
            return size.clone();
        }

        let mut font = self.base_font.clone();
        let (width, height) = (rect.width(), rect.height());
        loop {
            let galley = painter.layout(
                text.to_owned(),
                font.to_owned(),
                egui::Color32::WHITE,
                width,
            );
            if galley.rect.height() < height {
                break;
            } else {
                font.size *= self.downscale_factor;
            }
        }
        self.data.insert(text.to_owned(), font.clone());

        font
    }

    pub fn draw_fitted_text(&mut self, painter: &egui::Painter, text: &str, rect: egui::Rect) {
        let font = self.font(painter, text, rect);
        let (height, width) = (rect.height(), rect.width());
        let mut layout =
            egui::text::LayoutJob::simple(text.to_owned(), font, egui::Color32::WHITE, width);
        layout.halign = egui::Align::Center;
        let galley = painter.fonts(|f| f.layout_job(layout));
        let pos = rect.center_top() + egui::vec2(0.0, (height - galley.rect.height()) / 2.0);
        painter.galley(pos, galley);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}
