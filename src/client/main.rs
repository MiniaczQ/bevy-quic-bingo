#[path = "../common/mod.rs"]
mod common;
mod connecting;
mod game_ui;
mod menu_ui;
mod scoped;
mod states;
mod ui;
mod userdata;

use bevy::{
    prelude::*,
    window::{ExitCondition, WindowResolution},
};
use bevy_quinnet::shared::ClientId;
use connecting::ConnectionPlugin;
use game_ui::GameUiPlugin;
use menu_ui::MenuUiPlugin;
use scoped::ScopedExt;
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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Menu".into(),
                resizable: false,
                resolution: WindowResolution::new(400.0, 300.0),
                ..default()
            }),
            exit_condition: ExitCondition::OnPrimaryClosed,
            ..default()
        }))
        .add_plugins(MenuUiPlugin)
        .add_plugins(ConnectionPlugin)
        .add_plugins(GameUiPlugin)
        .add_state::<AppState>()
        .entity_scope::<AppState>()
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Name::new("camera"), Camera2dBundle::default()));
}
