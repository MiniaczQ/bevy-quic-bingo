#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../common/mod.rs"]
mod common;
mod connecting;
mod fit_text;
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
use states::StatesPlugin;
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
                resolution: WindowResolution::new(400.0, 600.0),
                ..default()
            }),
            exit_condition: ExitCondition::OnPrimaryClosed,
            ..default()
        }))
        .add_plugins(StatesPlugin)
        .add_plugins(ConnectionPlugin)
        .add_plugins(MenuUiPlugin)
        .add_plugins(GameUiPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
