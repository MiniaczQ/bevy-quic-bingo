use bevy::{
    audio::Volume,
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, PrimaryWindow, WindowRef, WindowResolution},
};
use bevy_egui::EguiContext;
use bevy_quinnet::client::Client;

use crate::{
    common::{
        bingo::{Board, BoardConfig, GameMode, WinCondition},
        protocol::{ClientMessage, ClientProps},
        teams::Team,
        BoardRes,
    },
    connecting::{StopConnection, TeamWon},
    states::AppState,
    ui::root_element,
    Clients,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), add_resources)
            .add_systems(
                OnExit(AppState::Playing),
                (remove_resources, remove_bingo_window),
            )
            .add_systems(
                Update,
                (
                    create_bingo_window,
                    game_menu_ui,
                    bingo_board_ui,
                    play_win_sfx,
                )
                    .run_if(in_state(AppState::Playing)),
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

#[derive(Debug, PartialEq, Eq)]
enum FlatWinCondition {
    InRow,
    Domination,
    FirstTo,
}

impl FlatWinCondition {
    pub fn flatten(value: WinCondition) -> Self {
        match value {
            WinCondition::InRow { length: _, rows: _ } => FlatWinCondition::InRow,
            WinCondition::Domination => FlatWinCondition::Domination,
            WinCondition::FirstTo(_) => FlatWinCondition::FirstTo,
        }
    }

    pub fn unflatten(self) -> WinCondition {
        match self {
            FlatWinCondition::InRow => WinCondition::InRow { length: 5, rows: 1 },
            FlatWinCondition::Domination => WinCondition::Domination,
            FlatWinCondition::FirstTo => WinCondition::FirstTo(13),
        }
    }
}

fn game_menu_ui(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut disconnect_events: EventWriter<StopConnection>,
    mut clients: ResMut<Clients>,
    client: Res<Client>,
    board: Res<BoardRes>,
    mut board_config: Local<Option<BoardConfig>>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };

    if board_config.is_none() {
        *board_config = Some(board.config.clone());
    }
    let board_conf = board_config.as_mut().unwrap();

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
            disconnect_events.send(StopConnection);
        }

        let self_props = clients.data.get_mut(&self_id).unwrap();
        if teams_selector(ui, self_props) {
            client
                .connection()
                .try_send_message(ClientMessage::ChangeTeam(self_props.team));
        }

        ui.separator();
        ui.label("Game Settings");
        ui.separator();
        ui.label(format!("Game mode: {}", board.config.mode.game_mode));
        ui.label(format!(
            "Win condition: {}",
            board.config.mode.win_condition
        ));

        if self_props.is_host {
            let mut mode_changed = false;
            let mut win_condition_changed = false;

            // Mode
            ui.horizontal(|ui| {
                mode_changed |= ui
                    .selectable_value(&mut board_conf.mode.game_mode, GameMode::FFA, "FFA")
                    .clicked();
                mode_changed |= ui
                    .selectable_value(&mut board_conf.mode.game_mode, GameMode::Lockout, "Lockout")
                    .clicked();
            });
            if mode_changed
                && board_conf.mode.game_mode != GameMode::Lockout
                && board_conf.mode.win_condition == WinCondition::Domination
            {
                board_conf.mode.win_condition = FlatWinCondition::InRow.unflatten();
                win_condition_changed = true;
            }

            // Win condition
            let mut flat_win_condition = FlatWinCondition::flatten(board_conf.mode.win_condition);
            ui.horizontal(|ui| {
                win_condition_changed |= ui
                    .selectable_value(
                        &mut flat_win_condition,
                        FlatWinCondition::InRow,
                        "N rows of M",
                    )
                    .clicked();
                if board_conf.mode.game_mode == GameMode::Lockout {
                    win_condition_changed |= ui
                        .selectable_value(
                            &mut flat_win_condition,
                            FlatWinCondition::Domination,
                            "Domination",
                        )
                        .clicked();
                }
                win_condition_changed |= ui
                    .selectable_value(
                        &mut flat_win_condition,
                        FlatWinCondition::FirstTo,
                        "First to N",
                    )
                    .clicked();
            });
            if win_condition_changed {
                board_conf.mode.win_condition = flat_win_condition.unflatten();
            }
            egui::Grid::new("Win Condition Grid").show(ui, |ui| {
                match &mut board_conf.mode.win_condition {
                    WinCondition::InRow {
                        ref mut length,
                        ref mut rows,
                    } => {
                        ui.label("Row length");
                        ui.add(egui::DragValue::new(length).speed(0.03)).changed();
                        ui.end_row();

                        ui.label("Row count");
                        ui.add(egui::DragValue::new(rows).speed(0.03)).changed();
                        ui.end_row();
                    }
                    WinCondition::Domination => {}
                    WinCondition::FirstTo(ref mut n) => {
                        ui.label("First to");
                        ui.add(egui::DragValue::new(n).speed(0.03)).changed();
                        ui.end_row();
                    }
                }
            });

            // Send update
            ui.horizontal(|ui| {
                let different_mode = board_conf.mode != board.config.mode;
                let different_prompts = board_conf.prompts != board.config.prompts;
                let different = different_mode || different_prompts;

                let restart = ui.button("Restart game").clicked();
                if restart {
                    if different {
                        if different_mode {
                            client
                                .connection()
                                .try_send_message(ClientMessage::SetMode(board_conf.mode.clone()));
                        }
                        if different_prompts {
                            client
                                .connection()
                                .try_send_message(ClientMessage::SetPrompts(
                                    board_conf.prompts.clone(),
                                ));
                        }
                    } else {
                        client
                            .connection()
                            .try_send_message(ClientMessage::ResetActivity);
                    }
                }

                let cancel = ui
                    .add_enabled(different, egui::Button::new("Cancel changes"))
                    .clicked();
                if cancel {
                    *board_conf = board.config.clone();
                }
            });
        }
    });
}

#[derive(Component)]
struct BingoWindow;

#[derive(Component)]
struct BingoWindowCamera;

fn create_bingo_window(
    mut commands: Commands,
    window: Query<Entity, With<BingoWindow>>,
    mut camera: Query<&mut Camera, With<BingoWindowCamera>>,
) {
    if window.get_single().is_ok() {
        return;
    }

    let window_id = commands
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

    let target = RenderTarget::Window(WindowRef::Entity(window_id));
    if let Ok(mut camera) = camera.get_single_mut() {
        camera.target = target;
    } else {
        commands.spawn(Camera2dBundle {
            camera: Camera {
                target,
                ..default()
            },
            ..default()
        });
    }
}

fn remove_bingo_window(
    mut commands: Commands,
    window: Query<Entity, With<BingoWindow>>,
    camera: Query<Entity, With<BingoWindowCamera>>,
) {
    if let Ok(window_id) = window.get_single() {
        commands.entity(window_id).despawn();
    }
    if let Ok(camera_id) = camera.get_single() {
        commands.entity(camera_id).despawn();
    }
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
    let mode = board.config.mode.game_mode;
    let activity = board.activity(x, y);

    let size = 50.0;
    let size2d = egui::Vec2::new(size, size);
    let mut widget = egui::Button::new(board.prompt(x, y)).rounding(0.0);
    match mode {
        GameMode::Lockout => {
            if let Some(team) = activity.iter().next() {
                widget = widget.fill(team.color());
            }
        }
        GameMode::FFA => {
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
        let mode = board.config.mode.game_mode;
        let win = board.check_win();
        let activity = board.activity_mut(x, y);
        let was_active = activity.contains(&team);
        let mut change = false;
        match was_active {
            true => {
                activity.remove(&team);
                change = true;
            }
            false => {
                if (mode != GameMode::Lockout || activity.is_empty()) && win.is_none() {
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

#[derive(Component)]
struct TeamWonSfx;

fn play_win_sfx(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut team_won: EventReader<TeamWon>,
    sfxs: Query<Entity, With<TeamWonSfx>>,
) {
    for _ in team_won.read() {
        for sfx in sfxs.iter() {
            commands.entity(sfx).despawn();
        }

        let source = asset_server.load("sfx/win.ogg");

        commands.spawn((
            TeamWonSfx,
            AudioBundle {
                source,
                settings: PlaybackSettings::ONCE.with_volume(Volume::new_relative(0.5)),
            },
        ));
    }
}
