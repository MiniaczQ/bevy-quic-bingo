#[path = "../common/mod.rs"]
mod common;
mod connecting;
mod game_ui;
mod menu_ui;
mod states;
mod teams;
mod util;

use bevy::prelude::*;
use bevy_quinnet::shared::ClientId;
use bevy_simple_text_input::TextInputPlugin;
use connecting::ConnectionPlugin;
use game_ui::GameUiPlugin;
use menu_ui::MenuUiPlugin;
use states::AppState;
use std::collections::HashMap;
use util::ScopedExt;

use common::protocol::ClientProps;

#[derive(Resource, Debug, Clone, Default)]
struct Clients {
    data: HashMap<ClientId, ClientProps>,
    self_id: ClientId,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TextInputPlugin)
        .add_plugins(MenuUiPlugin)
        .add_plugins(ConnectionPlugin)
        .add_plugins(GameUiPlugin)
        .insert_resource(Clients::default())
        .add_state::<AppState>()
        .entity_scope::<AppState>()
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Name::new("camera"), Camera2dBundle::default()));
}
