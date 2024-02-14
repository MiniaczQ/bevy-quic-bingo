use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, PrimaryWindow, WindowRef, WindowResolution},
};
use bevy_egui::EguiContext;
use bevy_quinnet::client::Client;

use crate::{
    common::{
        bingo::{Board, Mode, WinCondition},
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

#[derive(Debug, PartialEq, Eq)]
enum WinConditionFlat {
    InRow,
    Domination,
    FirstTo,
}

fn flatten_win_contition(value: WinCondition) -> WinConditionFlat {
    match value {
        WinCondition::InRow { length: _, rows: _ } => WinConditionFlat::InRow,
        WinCondition::Domination => WinConditionFlat::Domination,
        WinCondition::FirstTo(_) => WinConditionFlat::FirstTo,
    }
}

fn unflatten_win_contition(value: WinConditionFlat) -> WinCondition {
    match value {
        WinConditionFlat::InRow => WinCondition::InRow { length: 5, rows: 1 },
        WinConditionFlat::Domination => WinCondition::Domination,
        WinConditionFlat::FirstTo => WinCondition::FirstTo(13),
    }
}

fn game_menu_ui(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut disconnect_events: EventWriter<DisconnectEvent>,
    mut clients: ResMut<Clients>,
    client: Res<Client>,
    board: Res<BoardRes>,
    mut board_settings: Local<Option<BoardPrompts>>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };

    if board_settings.is_none() {
        *board_settings = Some(BoardPrompts::from_board(&board));
    }
    let board_settings = board_settings.as_mut().unwrap();

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

        ui.separator();
        ui.label("Game Settings");
        ui.separator();
        ui.label(format!("Mode: {:?}", board.mode));
        ui.label(format!("Win condition: {}", board.win_condition));

        if self_props.is_host {
            let mut mode_changed = false;
            let mut win_condition_changed = false;

            // Mode
            ui.horizontal(|ui| {
                mode_changed |= ui
                    .selectable_value(&mut board_settings.mode, Mode::FFA, "FFA")
                    .clicked();
                mode_changed |= ui
                    .selectable_value(&mut board_settings.mode, Mode::Lockout, "Lockout")
                    .clicked();
            });
            if mode_changed
                && board_settings.mode != Mode::Lockout
                && board_settings.win_condition == WinCondition::Domination
            {
                board_settings.win_condition = unflatten_win_contition(WinConditionFlat::InRow);
                win_condition_changed = true;
            }

            // Win condition
            let mut flat_win_condition = flatten_win_contition(board_settings.win_condition);
            ui.horizontal(|ui| {
                win_condition_changed |= ui
                    .selectable_value(
                        &mut flat_win_condition,
                        WinConditionFlat::InRow,
                        "N rows of M",
                    )
                    .clicked();
                if board_settings.mode == Mode::Lockout {
                    win_condition_changed |= ui
                        .selectable_value(
                            &mut flat_win_condition,
                            WinConditionFlat::Domination,
                            "Domination",
                        )
                        .clicked();
                }
                win_condition_changed |= ui
                    .selectable_value(
                        &mut flat_win_condition,
                        WinConditionFlat::FirstTo,
                        "First to N",
                    )
                    .clicked();
            });
            if win_condition_changed {
                board_settings.win_condition = unflatten_win_contition(flat_win_condition);
            }
            egui::Grid::new("Win Condition Grid").show(ui, |ui| {
                match &mut board_settings.win_condition {
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

            ui.horizontal(|ui| {
                let different = !board_settings.same_as_board(&board);
                let restart = ui
                    .add_enabled(different, egui::Button::new("Restart game"))
                    .clicked();
                if restart {
                    client
                        .connection()
                        .try_send_message(ClientMessage::UpdateBoard(board_settings.clone()));
                }

                let cancel = ui
                    .add_enabled(different, egui::Button::new("Cancel changes"))
                    .clicked();
                if cancel {
                    *board_settings = BoardPrompts::from_board(&board);
                }
            });
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
                if (mode != Mode::Lockout || activity.is_empty()) && win.is_none() {
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
