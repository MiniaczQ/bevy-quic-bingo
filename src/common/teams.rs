use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Team {
    Red,
    Green,
    Blue,
    Cyan,
    Pink,
    Magenta,
    Purple,
    Yellow,
}

impl Team {
    pub fn color(&self) -> Color32 {
        match self {
            Team::Red => Color32::RED,
            Team::Green => Color32::GREEN,
            Team::Blue => Color32::BLUE,
            Team::Cyan => Color32::from_rgb(0, 183, 235),
            Team::Pink => Color32::from_rgb(255, 105, 180),
            Team::Magenta => Color32::from_rgb(255, 0, 144),
            Team::Purple => Color32::from_rgb(186, 85, 211),
            Team::Yellow => Color32::YELLOW,
        }
    }
}
