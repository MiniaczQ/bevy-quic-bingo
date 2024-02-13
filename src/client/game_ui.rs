use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, PrimaryWindow, WindowRef, WindowResolution},
};
use bevy_egui::EguiContext;
use bevy_quinnet::client::Client;

use crate::{
    common::{
        bingo::Board,
        protocol::{ClientMessage, ClientProps},
        teams::Team,
        BoardRes,
    },
    connecting::DisconnectEvent,
    states::AppState,
    ui::root_element,
    Clients,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Playing),
            (add_resources, create_bingo_window),
        )
        .add_systems(
            OnExit(AppState::Playing),
            (remove_resources, remove_bingo_window),
        )
        .add_systems(
            Update,
            (game_menu_ui, bingo_board_ui).run_if(in_state(AppState::Playing)),
        );
    }
}

fn add_resources(mut commands: Commands) {
    commands.insert_resource(BoardRes::default());
    commands.insert_resource(Clients::default());
}

fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<BoardRes>();
    commands.remove_resource::<Clients>();
}

fn team_to_ui(ui: &mut egui::Ui, value: &mut Option<Team>, team: Option<Team>) -> egui::Response {
    let label = match team {
        Some(team) => egui::RichText::new(format!("{:?}", team)).color(team.color()),
        None => egui::RichText::new("Spectator"),
    };
    ui.selectable_value(value, team, label)
}

fn teams_selector(ui: &mut egui::Ui, client_props: &mut ClientProps) -> bool {
    let mut clicked = false;
    ui.vertical(|ui| {
        clicked |= ui
            .vertical_centered_justified(|ui| team_to_ui(ui, &mut client_props.team, None))
            .inner
            .clicked();

        egui::Grid::new("Game Team Grid").show(ui, |ui| {
            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Blue)).clicked();
            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Cyan)).clicked();
            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Green)).clicked();
            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Magenta)).clicked();
            ui.end_row();

            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Pink)).clicked();
            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Purple)).clicked();
            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Red)).clicked();
            clicked |= team_to_ui(ui, &mut client_props.team, Some(Team::Yellow)).clicked();
            ui.end_row();
        });
    });
    clicked
}

fn game_menu_ui(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut disconnect_events: EventWriter<DisconnectEvent>,
    mut clients: ResMut<Clients>,
    client: Res<Client>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };
    let self_id = clients.self_id;
    let Some(client_props) = clients.data.get_mut(&self_id) else {
        return;
    };

    root_element(ctx.get_mut(), |ui| {
        egui::Grid::new("Game Menu Grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Connected as:");
                ui.label(&client_props.username);
                ui.end_row();
            });

        if teams_selector(ui, client_props) {
            client
                .connection()
                .try_send_message(ClientMessage::ChangeTeam(client_props.team));
        }

        ui.horizontal_centered(|ui| {
            let disconnect = ui.button("Disconnect").clicked();
            if disconnect {
                disconnect_events.send(DisconnectEvent);
            }
        })
    });
}

#[derive(Component)]
struct BingoWindow;

fn create_bingo_window(mut commands: Commands) {
    let second_window_id = commands
        .spawn((
            BingoWindow,
            Window {
                title: "Bingo Board".to_owned(),
                resolution: WindowResolution::new(270.0, 270.0),
                present_mode: PresentMode::AutoVsync,
                resizable: false,
                ..default()
            },
        ))
        .id();

    commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(second_window_id)),
            ..default()
        },
        ..default()
    });
}

fn remove_bingo_window(mut commands: Commands, query: Query<Entity, With<BingoWindow>>) {
    let entity = query.single();
    commands.entity(entity).despawn_recursive();
}

fn bingo_board_ui(
    mut egui_ctx: Query<&mut EguiContext, Without<PrimaryWindow>>,
    mut board: ResMut<BoardRes>,
    clients: Res<Clients>,
    client: Res<Client>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };
    let self_id = clients.self_id;
    let Some(client_props) = clients.data.get(&self_id) else {
        return;
    };

    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        egui::Grid::new("Bingo Grid")
            .spacing(egui::Vec2::new(1.0, 1.0))
            .show(ui, |ui| {
                for x in 0..5 {
                    for y in 0..5 {
                        add_bingo_field(ui, &mut board, &client_props, &client, (x, y));
                    }
                    ui.end_row();
                }
            });
    });
}

fn add_bingo_field(
    ui: &mut egui::Ui,
    board: &mut Board,
    client_props: &ClientProps,
    client: &Client,
    (x, y): (u8, u8),
) {
    let pos = ui.next_widget_position();
    let clicked = ui
        .add_sized(
            egui::Vec2::new(50.0, 50.0),
            egui::Button::new(board.prompt(x, y)),
        )
        .clicked();
    ui.add(egui::)

    if client_props.team.is_some() && clicked {
        let team = client_props.team.unwrap();
        let activity = board.activity_mut(x, y);
        let is_active = activity.contains(&team);
        match !is_active {
            true => activity.remove(&team),
            false => activity.insert(team),
        };

        client
            .connection()
            .try_send_message(ClientMessage::UpdateActivity {
                team,
                x,
                y,
                is_active: !is_active,
            });
    }
}
