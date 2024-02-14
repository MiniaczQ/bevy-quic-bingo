use std::net::SocketAddr;

use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiPlugin};

use crate::{
    connecting::StartConnection,
    states::AppState,
    ui::root_element,
    userdata::{Userdata, UserdataPlugin},
};

pub struct MenuUiPlugin;

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(UserdataPlugin)
            .add_systems(Update, main_menu_ui.run_if(in_state(AppState::MainMenu)));
    }
}

fn validate_username(username: &str) -> bool {
    if username.len() < 4 || username.len() > 32 {
        return false;
    }
    for c in username.chars() {
        if !c.is_alphanumeric() {
            return false;
        }
    }
    true
}

fn add_validated_textbox(
    ui: &mut egui::Ui,
    id_valid: bool,
    buffer: &mut dyn egui::TextBuffer,
) -> egui::Response {
    ui.add(egui::TextEdit::singleline(buffer).text_color(if id_valid {
        egui::Color32::WHITE
    } else {
        egui::Color32::RED
    }))
}

fn main_menu_ui(
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut app_exit: EventWriter<AppExit>,
    mut userdata: ResMut<Userdata>,
    mut client_connect: EventWriter<StartConnection>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };

    let addr: Option<SocketAddr> = userdata.addr.parse().ok();
    let valid_username = validate_username(&userdata.username);

    root_element(ctx.get_mut(), |ui| {
        egui::Grid::new("Main Menu Grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Username:");
                add_validated_textbox(ui, valid_username, &mut userdata.username)
                    .on_hover_text("4-32 alphanumerics");
                ui.end_row();

                ui.label("Address:");
                add_validated_textbox(ui, addr.is_some(), &mut userdata.addr);
                ui.end_row();
            });

        ui.vertical_centered(|ui| {
            let connect = ui
                .add_enabled(
                    addr.is_some() && valid_username,
                    egui::Button::new("Connect"),
                )
                .clicked();
            if connect {
                client_connect.send(StartConnection {
                    username: userdata.username.clone(),
                    addr: addr.unwrap(),
                });
            }

            let exit = ui.button("Exit").clicked();
            if exit {
                app_exit.send(AppExit);
            }
        });
    });
}
