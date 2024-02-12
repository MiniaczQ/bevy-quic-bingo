use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_quinnet::client::Client;

use crate::{
    common::{protocol::ClientMessage, BoardRes},
    states::AppState,
    util::scoped::Scoped,
    Clients,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), (add_board, setup_ui).chain())
            .add_systems(OnExit(AppState::Playing), remove_board)
            .add_systems(Update, ui_button_system.run_if(in_state(AppState::Playing)));
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn add_board(mut commands: Commands) {
    commands.insert_resource(BoardRes::default());
}

fn remove_board(mut commands: Commands) {
    commands.remove_resource::<BoardRes>();
}

fn ui_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &GridElement,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    client: ResMut<Client>,
    clients: ResMut<Clients>,
) {
    for (interaction, mut color, mut border_color, element) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                if let Some(team) = clients.data.get(&clients.self_id).unwrap().team {
                    client
                        .connection()
                        .send_message(ClientMessage::UpdateActivity {
                            team,
                            x: element.0,
                            y: element.1,
                            is_active: true,
                        })
                        .unwrap();
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root = commands
        .spawn((
            Scoped(AppState::Playing),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    spawn_grid_with_size(&mut commands, &asset_server, (5, 5)).set_parent(root);
}

#[derive(Component)]
struct GridElement(u8, u8);

fn spawn_grid_with_size<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    size: (u8, u8),
) -> EntityCommands<'w, 's, 'a> {
    let grid = commands
        .spawn((NodeBundle {
            style: Style {
                display: Display::Grid,
                width: Val::VMin(100.0),
                height: Val::VMin(100.0),
                grid_template_columns: vec![GridTrack::flex(1.0); size.0 as usize],
                grid_template_rows: vec![GridTrack::flex(1.0); size.1 as usize],
                ..default()
            },
            ..default()
        },))
        .id();
    for x in 0..size.0 {
        for y in 0..size.1 {
            commands
                .spawn((
                    GridElement(x, y),
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button button button",
                        TextStyle {
                            font: asset_server.load("fonts/UbuntuMono-R.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                })
                .set_parent(grid);
        }
    }
    commands.entity(grid)
}
