use bevy::prelude::*;
use bevy_quinnet::shared::AsyncRuntime;
use serde::{Deserialize, Serialize};

use crate::{states::AppState, storage::Storage};

pub struct UserdataPlugin;

impl Plugin for UserdataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), insert_userdata)
            .add_systems(OnExit(AppState::Playing), remove_userdata);
    }
}

pub const USERDATA_PATH: &str = "userdata.toml";

fn insert_userdata(mut commands: Commands, runtime: Res<AsyncRuntime>) {
    let mut userdata = Storage::<Userdata>::new(runtime.handle().clone());
    userdata.queue_load(USERDATA_PATH);
    commands.insert_resource(userdata);
}

fn remove_userdata(mut commands: Commands) {
    commands.remove_resource::<Storage<Userdata>>();
}

#[derive(Resource, Serialize, Deserialize, Default, Clone)]
pub struct Userdata {
    pub username: String,
    pub addr: String,
}
