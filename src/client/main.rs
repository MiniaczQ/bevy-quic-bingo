#[path = "../common/mod.rs"]
mod common;
mod connecting;
mod game_ui;
mod menu_ui;
mod states;
mod teams;
mod util;

use bevy::prelude::*;
use bevy_quinnet::{client::QuinnetClientPlugin, shared::ClientId};
use game_ui::GameUiPlugin;
use menu_ui::MenuUiPlugin;
use states::AppState;
use std::collections::HashMap;

use common::protocol::ClientProps;

#[derive(Resource, Debug, Clone, Default)]
struct Clients {
    data: HashMap<ClientId, ClientProps>,
    self_id: ClientId,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(QuinnetClientPlugin::default())
        .add_plugins(MenuUiPlugin)
        .add_plugins(GameUiPlugin)
        .insert_resource(Clients::default())
        .add_state::<AppState>()
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Name::new("camera"), Camera2dBundle::default()));
}
