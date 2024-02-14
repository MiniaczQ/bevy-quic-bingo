use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, PrimaryWindow, WindowRef, WindowResolution},
};
use bevy_egui::EguiContext;
use bevy_quinnet::client::Client;

use crate::{
    common::{
        bingo::{Board, Mode},
        protocol::{BoardPrompts, ClientMessage, ClientProps},
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
    ui.separator();
    ui.label("Team");
    ui.separator();

    clicked |= ui
        .vertical_centered_justified(|ui| team_to_ui(ui, &mut client_props.team, None))
        .inner
        .clicked();

    egui::Grid::new("Game Team Grid").show(ui, |ui| {
        let mut iter = Team::iter();
        for _ in 0..2 {
            for _ in 0..4 {
                clicked |= team_to_ui(
                    ui,
                    &mut client_props.team,
                    Some(iter.next().unwrap().clone()),
                )
                .clicked();
            }
            ui.end_row()
        }
    });
    clicked
}

const HOST_ICON: &str = "★";
const KICK_ICON: &str = "🗑";

fn game_menu_ui(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut disconnect_events: EventWriter<DisconnectEvent>,
    mut clients: ResMut<Clients>,
    client: Res<Client>,
    mut board: ResMut<BoardRes>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };

    root_element(ctx.get_mut(), |ui| {
        let self_id = clients.self_id;
        let Some(self_props) = clients.data.get(&self_id) else {
            return;
        };

        ui.separator();
        ui.label("Users");
        ui.separator();

        egui::Grid::new("Game Team Grid").show(ui, |ui| {
            for (client_id, client_props) in clients.data.iter() {
                if client_props.is_host {
                    ui.label(HOST_ICON);
                } else if self_props.is_host {
                    let kick = ui.small_button(KICK_ICON).clicked();
                    if kick {
                        client
                            .connection()
                            .try_send_message(ClientMessage::Kick(*client_id));
                    }
                } else {
                    ui.label("");
                }
                let mut username = egui::RichText::new(&client_props.username);
                if self_id == *client_id {
                    username = username.strong()
                }
                if let Some(team) = client_props.team {
                    username = username.color(team.color());
                }
                ui.label(username);
                ui.end_row();
            }
        });

        let disconnect = ui.button("Disconnect").clicked();
        if disconnect {
            disconnect_events.send(DisconnectEvent);
        }

        let self_props = clients.data.get_mut(&self_id).unwrap();
        if teams_selector(ui, self_props) {
            client
                .connection()
                .try_send_message(ClientMessage::ChangeTeam(self_props.team));
        }

        if self_props.is_host {
            ui.separator();
            ui.label("Game mode");
            ui.separator();

            ui.horizontal(|ui| {
                let mut changed = false;
                changed |= ui
                    .selectable_value(&mut board.mode, Mode::FFA, "FFA")
                    .clicked();
                changed |= ui
                    .selectable_value(&mut board.mode, Mode::Lockout, "Lockout")
                    .clicked();
                if changed {
                    client
                        .connection()
                        .try_send_message(ClientMessage::UpdateBoard(BoardPrompts {
                            mode: board.mode,
                            x_size: board.x_size,
                            y_size: board.y_size,
                            prompts: board.prompts.clone(),
                        }));
                }
            });

            let reset = ui.button("Reset Board").clicked();
            if reset {
                client
                    .connection()
                    .try_send_message(ClientMessage::ResetActivity);
            }
        } else {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Game mode");
                ui.label(format!("{:?}", board.mode));
            });
            ui.separator();
        }
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
    let team = client_props.team;
    let mode = board.mode;
    let activity = board.activity(x, y);

    let size = 50.0;
    let size2d = egui::Vec2::new(size, size);
    let mut widget = egui::Button::new(board.prompt(x, y)).rounding(0.0);
    match mode {
        Mode::Lockout => {
            if let Some(team) = activity.iter().next() {
                widget = widget.fill(team.color());
            }
        }
        Mode::FFA => {
            if let Some(team) = team {
                if activity.contains(&team) {
                    widget = widget.fill(team.color());
                }
            }
        }
    }

    let button = ui.add_sized(size2d, widget);
    let clicked = button.clicked();

    let painter = ui.painter_at(button.rect);
    let pos = button.rect.left_top();
    let size = button.rect.size();
    let (x_step, y_step) = (size.x / 4.0, size.y / 4.0);
    for (i, team) in Team::iter().enumerate() {
        if activity.contains(&team) {
            let x_offset = (i % 4) as f32 * x_step;
            let y_offset = (i / 4) as f32 * y_step * 3.0;
            let pos1 = pos + egui::Vec2::new(x_offset, y_offset);
            let pos2 = pos1 + egui::Vec2::new(x_step, y_step);
            painter.rect_filled(egui::Rect::from_two_pos(pos1, pos2), 0.0, team.color());
        }
    }

    if client_props.team.is_some() && clicked {
        let team = client_props.team.unwrap();
        let mode = board.mode;
        let activity = board.activity_mut(x, y);
        let was_active = activity.contains(&team);
        let mut change = false;
        match was_active {
            true => {
                activity.remove(&team);
                change = true;
            }
            false => {
                if mode != Mode::Lockout || activity.is_empty() {
                    activity.insert(team);
                    change = true;
                }
            }
        };

        if change {
            client
                .connection()
                .try_send_message(ClientMessage::UpdateActivity {
                    team,
                    x,
                    y,
                    is_active: !was_active,
                });
        }
    }
}
