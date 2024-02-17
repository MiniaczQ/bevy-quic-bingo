use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    states::AppState,
    storage::{Storage, StoragePath},
};

pub struct UserdataPlugin;

impl Plugin for UserdataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), insert_userdata)
            .add_systems(OnExit(AppState::Playing), remove_userdata);
    }
}

fn insert_userdata(mut commands: Commands) {
    commands.init_resource::<Storage<Userdata>>();
}

fn remove_userdata(mut commands: Commands) {
    commands.remove_resource::<Storage<Userdata>>();
}

#[derive(Resource, Serialize, Deserialize, Default, Clone)]
pub struct Userdata {
    pub username: String,
    pub addr: String,
}

impl StoragePath for Userdata {
    fn path() -> impl AsRef<std::path::Path> + Send + 'static {
        "userdata.toml"
    }
}
