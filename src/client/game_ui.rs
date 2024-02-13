use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, PrimaryWindow, WindowRef, WindowResolution},
};
use bevy_egui::EguiContext;

use crate::{common::BoardRes, states::AppState};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), (add_board, create_bingo_window))
            .add_systems(OnExit(AppState::Playing), remove_board)
            .add_systems(Update, bingo_board_ui.run_if(in_state(AppState::Playing)));
    }
}

fn add_board(mut commands: Commands) {
    commands.insert_resource(BoardRes::default());
}

fn remove_board(mut commands: Commands) {
    commands.remove_resource::<BoardRes>();
}

fn create_bingo_window(mut commands: Commands) {
    let second_window_id = commands
        .spawn(Window {
            title: "Bingo Board".to_owned(),
            resolution: WindowResolution::new(250.0, 250.0),
            present_mode: PresentMode::AutoVsync,
            resizable: false,
            ..default()
        })
        .id();

    commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(second_window_id)),
            ..default()
        },
        ..default()
    });
}

fn bingo_board_ui(mut egui_ctx: Query<&mut EguiContext, Without<PrimaryWindow>>) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };
    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        egui::Grid::new("Bingo Grid").show(ui, |ui| {
            for x in 0..5 {
                for y in 0..5 {
                    let (id, rect) = ui.allocate_space(egui::Vec2::new(50.0, 50.0));
                    let btn = ui.interact(rect, id, egui::Sense::click());
                    if btn.clicked() {
                        info!("{} {}", x, y);
                    }
                }
                ui.end_row();
            }
        });
    });
}
