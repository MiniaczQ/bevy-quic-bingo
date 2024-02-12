use bevy::prelude::*;

#[derive(States, PartialEq, Hash, Default, Debug, Eq, Clone)]
pub enum AppState {
    #[default]
    Menu,
    Connecting,
    Playing,
}
