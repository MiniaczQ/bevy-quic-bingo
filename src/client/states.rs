use bevy::prelude::*;

use crate::scoped::ScopedExt;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>().entity_scope::<AppState>();
    }
}

#[derive(States, PartialEq, Hash, Default, Debug, Eq, Clone)]
pub enum AppState {
    #[default]
    MainMenu,
    Playing,
}
