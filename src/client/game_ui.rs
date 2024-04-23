use bevy::{
    audio::Volume,
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, PrimaryWindow, WindowRef, WindowResolution},
};
use bevy_egui::EguiContext;
use bevy_quinnet::client::Client;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use common::{
    bingo::{Board, BoardPrompts, GameMode, WinCondition},
    protocol::{ClientMessage, ClientProps},
    teams::Team,
    BoardRes, ConfMode, ConfPrompts,
};

use crate::{
    connecting::{StopConnection, TeamWon},
    fit_text::PromptLayoutCache,
    scoped::Scoped,
    states::AppState,
    storage::{Storage, StoragePath},
    ui::root_element,
    Clients,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), add_resources)
            .add_systems(OnExit(AppState::Playing), remove_resources)
            .add_systems(
                Update,
                (
                    create_bingo_window,
                    game_menu_ui,
                    bingo_board_ui,
                    play_win_sfx,
                    resize_window,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn add_resources(mut commands: Commands) {
    commands.init_resource::<BoardRes>();
    commands.init_resource::<Clients>();
    commands.init_resource::<ConfMode>();
    commands.init_resource::<ConfPrompts>();
    commands.init_resource::<PromptLayoutCache>();
    commands.init_resource::<Storage<PromptsString>>();
}

fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<BoardRes>();
    commands.remove_resource::<Clients>();
    commands.remove_resource::<ConfMode>();
    commands.remove_resource::<ConfPrompts>();
    commands.remove_resource::<PromptLayoutCache>();
    commands.remove_resource::<Storage<PromptsString>>();
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
                clicked |=
                    team_to_ui(ui, &mut client_props.team, Some(*iter.next().unwrap())).clicked();
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

#[derive(Clone, Serialize, Deserialize, Default)]
struct PromptsString {
    prompts: String,
}

impl StoragePath for PromptsString {
    fn path() -> impl AsRef<std::path::Path> + Send + 'static {
        "prompts.toml"
    }
}

#[allow(clippy::too_many_arguments)]
fn game_menu_ui(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut disconnect_events: EventWriter<StopConnection>,
    mut clients: ResMut<Clients>,
    client: Res<Client>,
    board: Res<BoardRes>,
    mut mode_conf: ResMut<ConfMode>,
    mut prompts_conf: ResMut<ConfPrompts>,
    mut prompts_str_storage: ResMut<Storage<PromptsString>>,
    mut cache: ResMut<PromptLayoutCache>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };
    let Some(prompts_str) = prompts_str_storage.get() else {
        return;
    };
    let mut prompt_str_changed = false;

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
            // Mode
            let mut mode_game_mode_changed = false;
            ui.horizontal(|ui| {
                mode_game_mode_changed |= ui
                    .selectable_value(&mut mode_conf.game_mode, GameMode::FFA, "FFA")
                    .clicked();
                mode_game_mode_changed |= ui
                    .selectable_value(&mut mode_conf.game_mode, GameMode::Lockout, "Lockout")
                    .clicked();
            });

            // Win condition
            let mut mode_win_condition_changed = false;
            if mode_game_mode_changed
                && mode_conf.game_mode != GameMode::Lockout
                && mode_conf.win_condition == WinCondition::Domination
            {
                mode_conf.win_condition = FlatWinCondition::InRow.unflatten();
                mode_win_condition_changed = true;
            }
            let mut flat_win_condition = FlatWinCondition::flatten(mode_conf.win_condition);
            ui.horizontal(|ui| {
                mode_win_condition_changed |= ui
                    .selectable_value(
                        &mut flat_win_condition,
                        FlatWinCondition::InRow,
                        "N rows of M",
                    )
                    .clicked();
                if mode_conf.game_mode == GameMode::Lockout {
                    mode_win_condition_changed |= ui
                        .selectable_value(
                            &mut flat_win_condition,
                            FlatWinCondition::Domination,
                            "Domination",
                        )
                        .clicked();
                }
                mode_win_condition_changed |= ui
                    .selectable_value(
                        &mut flat_win_condition,
                        FlatWinCondition::FirstTo,
                        "First to N",
                    )
                    .clicked();
            });
            if mode_win_condition_changed {
                mode_conf.win_condition = flat_win_condition.unflatten();
            }
            egui::Grid::new("Win Condition Grid").show(ui, |ui| {
                match &mut mode_conf.win_condition {
                    WinCondition::InRow {
                        ref mut length,
                        ref mut rows,
                    } => {
                        ui.label("Row length");
                        mode_win_condition_changed |=
                            ui.add(egui::DragValue::new(length).speed(0.03)).changed();
                        ui.end_row();

                        ui.label("Row count");
                        mode_win_condition_changed |=
                            ui.add(egui::DragValue::new(rows).speed(0.03)).changed();
                        ui.end_row();
                    }
                    WinCondition::Domination => {}
                    WinCondition::FirstTo(ref mut n) => {
                        ui.label("First to");
                        mode_win_condition_changed |=
                            ui.add(egui::DragValue::new(n).speed(0.03)).changed();
                        ui.end_row();
                    }
                }
            });
            mode_conf.changed |= mode_game_mode_changed || mode_win_condition_changed;

            let mut prompts_size_changed = false;
            egui::Grid::new("Bingo Size Grid").show(ui, |ui| {
                ui.label("Board width");
                prompts_size_changed |= ui
                    .add(
                        egui::DragValue::new(&mut prompts_conf.x_size)
                            .speed(0.03)
                            .clamp_range(1..=255),
                    )
                    .changed();
                ui.end_row();
                ui.label("Board height");
                prompts_size_changed |= ui
                    .add(
                        egui::DragValue::new(&mut prompts_conf.y_size)
                            .speed(0.03)
                            .clamp_range(1..=255),
                    )
                    .changed();
                ui.end_row();
            });

            let randomize = ui.button("Randomize prompts").clicked();
            if randomize | prompts_size_changed {
                let mut prompts = prompts_str
                    .prompts
                    .split('\n')
                    .filter_map(|x| {
                        let x = x.trim();
                        if x.is_empty() {
                            None
                        } else {
                            Some(x.to_owned())
                        }
                    })
                    .collect::<Vec<_>>();
                let prompt_count = prompts.len();
                let target_prompt_count =
                    prompts_conf.x_size as usize * prompts_conf.y_size as usize;
                if target_prompt_count > prompt_count {
                    prompts.extend(vec![String::new(); target_prompt_count - prompt_count]);
                }
                prompts.shuffle(&mut rand::thread_rng());
                prompts.truncate(target_prompt_count);
                prompts_conf.prompts.prompts = prompts;
                cache.clear();
            }
            prompts_conf.changed |= prompts_size_changed || randomize;

            // Send update
            ui.horizontal(|ui| {
                let different = mode_conf.changed || prompts_conf.changed;

                let restart = ui
                    .add_enabled(
                        different || board.activity.activity.iter().any(|x| !x.is_empty()),
                        egui::Button::new("Restart game"),
                    )
                    .clicked();
                if restart {
                    if different {
                        if mode_conf.changed {
                            client
                                .connection()
                                .try_send_message(ClientMessage::SetMode(mode_conf.clone()));
                        }
                        if prompts_conf.changed {
                            client
                                .connection()
                                .try_send_message(ClientMessage::SetPrompts(prompts_conf.clone()));
                            prompts_conf.changed = false;
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
                    **mode_conf = board.config.mode.clone();
                    mode_conf.changed = false;
                    **prompts_conf = board.config.prompts.clone();
                    prompts_conf.changed = false;
                }
            });

            ui.separator();
            ui.label("Prompts");
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    prompt_str_changed |= ui.text_edit_multiline(&mut prompts_str.prompts).changed()
                });
        }
    });

    if prompt_str_changed {
        prompts_str_storage.queue_save();
    }
}

#[derive(Component)]
struct BingoWindow;

#[derive(Component)]
struct BingoWindowCamera;

fn size_from_board(prompts: &BoardPrompts) -> (f32, f32) {
    let y_size = prompts.y_size as f32;
    let height = y_size * FIELD_SIZE + (y_size + 1.0) * GAP_SIZE;

    let x_size = prompts.x_size as f32;
    let width = x_size * FIELD_SIZE + (x_size + 1.0) * GAP_SIZE;

    (width, height)
}

fn create_bingo_window(
    mut commands: Commands,
    window: Query<Entity, With<BingoWindow>>,
    board: Res<BoardRes>,
    mut camera: Query<&mut Camera, With<BingoWindowCamera>>,
) {
    if window.get_single().is_ok() {
        return;
    }

    let (width, height) = size_from_board(&board.config.prompts);

    let window_id = commands
        .spawn((
            Scoped(AppState::Playing),
            BingoWindow,
            Window {
                title: "Bingo Board".to_owned(),
                resolution: WindowResolution::new(width, height),
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
        commands.spawn((
            Scoped(AppState::Playing),
            Camera2dBundle {
                camera: Camera {
                    target,
                    ..default()
                },
                ..default()
            },
        ));
    }
}

pub const FIELD_SIZE: f32 = 120.0;
pub const GAP_SIZE: f32 = 3.0;

fn bingo_board_ui(
    mut egui_ctx: Query<&mut EguiContext, Without<PrimaryWindow>>,
    mut board: ResMut<BoardRes>,
    prompts_conf: Res<ConfPrompts>,
    clients: Res<Clients>,
    client: Res<Client>,
    mut prompt_layout_cache: ResMut<PromptLayoutCache>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };
    let self_id = clients.self_id;
    let Some(client_props) = clients.data.get(&self_id) else {
        return;
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(GAP_SIZE))
        .show(ctx.get_mut(), |ui| {
            egui::Grid::new("Bingo Grid")
                .spacing((GAP_SIZE, GAP_SIZE))
                .show(ui, |ui| {
                    ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
                    ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.active.expansion = 0.0;

                    if prompts_conf.changed {
                        for y in 0..prompts_conf.y_size {
                            for x in 0..prompts_conf.x_size {
                                preview_bingo_field(
                                    ui,
                                    &prompts_conf,
                                    (x, y),
                                    &mut prompt_layout_cache,
                                );
                            }
                            ui.end_row();
                        }
                    } else {
                        for y in 0..board.config.prompts.y_size {
                            for x in 0..board.config.prompts.x_size {
                                playable_bingo_field(
                                    ui,
                                    &mut board,
                                    client_props,
                                    &client,
                                    (x, y),
                                    &mut prompt_layout_cache,
                                );
                            }
                            ui.end_row();
                        }
                    }
                });
        });
}

fn playable_bingo_field(
    ui: &mut egui::Ui,
    board: &mut Board,
    client_props: &ClientProps,
    client: &Client,
    (x, y): (u8, u8),
    prompt_layout_cache: &mut PromptLayoutCache,
) {
    let team = client_props.team;
    let mode = board.config.mode.game_mode;
    let activity = board.activity(x, y);
    let mut widget = egui::Button::new("").rounding(0.0);
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

    let size = egui::Vec2::new(FIELD_SIZE, FIELD_SIZE);
    let button = ui.add_sized(size, widget);
    let clicked = button.clicked();

    let pos = button.rect.left_top();
    let size = button.rect.size();
    let (x_step, y_step) = (size.x / 4.0, size.y / 4.0);
    let painter = ui.painter_at(button.rect);

    prompt_layout_cache.draw_fitted_text(
        &painter,
        board.prompt(x, y),
        egui::Rect::from_min_size(
            pos + egui::vec2(0.0, y_step),
            egui::vec2(size.x, size.y / 2.0),
        ),
    );

    for (i, team) in Team::iter().enumerate() {
        if activity.contains(team) {
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

fn preview_bingo_field(
    ui: &mut egui::Ui,
    prompts: &BoardPrompts,
    (x, y): (u8, u8),
    prompt_layout_cache: &mut PromptLayoutCache,
) {
    let widget = egui::Button::new("").rounding(0.0);
    let size = egui::Vec2::new(FIELD_SIZE, FIELD_SIZE);
    let button = ui.add_sized(size, widget);
    let pos = button.rect.left_top();
    let size = button.rect.size();
    let y_step = size.y / 4.0;
    let painter = ui.painter_at(button.rect);

    prompt_layout_cache.draw_fitted_text(
        &painter,
        prompts.prompt(x, y),
        egui::Rect::from_min_size(
            pos + egui::vec2(0.0, y_step),
            egui::vec2(size.x, size.y / 2.0),
        ),
    );
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
            Scoped(AppState::Playing),
            TeamWonSfx,
            AudioBundle {
                source,
                settings: PlaybackSettings::ONCE.with_volume(Volume::new_relative(0.5)),
            },
        ));
    }
}

fn resize_window(
    mut window: Query<&mut Window, With<BingoWindow>>,
    board: Res<BoardRes>,
    prompts_conf: Res<ConfPrompts>,
) {
    if let Ok(mut window) = window.get_single_mut() {
        let prompts = match prompts_conf.changed {
            true => &prompts_conf,
            false => &board.config.prompts,
        };
        let (width, height) = size_from_board(prompts);

        let (window_width, window_height) = (window.resolution.width(), window.resolution.height());
        if width != window_width || height != window_height {
            window.resolution.set(width, height)
        }
    }
}
