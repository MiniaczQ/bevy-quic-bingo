use std::{net::SocketAddr, str::FromStr, thread::sleep, time::Duration};

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode, connection::ConnectionConfiguration, Client,
    QuinnetClientPlugin,
};

use crate::{
    common::{
        protocol::{ClientMessage, ServerMessage},
        BoardRes,
    },
    states::AppState,
    Clients,
};

#[derive(Event)]
pub struct ConnectEvent {
    pub username: String,
    pub addr: SocketAddr,
}

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectEvent>()
            .add_event::<DisconnectEvent>()
            .add_plugins(QuinnetClientPlugin::default())
            .add_systems(
                Update,
                start_connection.run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(
                Update,
                handle_server_messages.run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (on_app_exit, on_disconnect).run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Event)]
pub struct DisconnectEvent;

pub fn on_disconnect(
    mut events: EventReader<DisconnectEvent>,
    mut client: ResMut<Client>,
    mut state: ResMut<NextState<AppState>>,
) {
    for _ in events.read() {
        client
            .connection()
            .send_message(ClientMessage::Disconnect {})
            .unwrap();
        state.set(AppState::MainMenu);
        sleep(Duration::from_secs_f32(0.1));
        client.close_all_connections().unwrap();
    }
}

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, mut client: ResMut<Client>) {
    if !app_exit_events.is_empty() {
        client
            .connection()
            .send_message(ClientMessage::Disconnect {})
            .unwrap();
        sleep(Duration::from_secs_f32(0.1));
        client.close_all_connections().unwrap();
    }
}

fn handle_server_messages(
    mut clients: ResMut<Clients>,
    mut client: ResMut<Client>,
    mut board: ResMut<BoardRes>,
    mut events: EventWriter<DisconnectEvent>,
) {
    loop {
        let result = client.connection_mut().receive_message::<ServerMessage>();

        match result {
            Ok(Some(msg)) => match msg {
                ServerMessage::InitClient(self_id) => {
                    clients.self_id = self_id;
                    info!("Connected with id {}", self_id);
                }
                ServerMessage::UpdateClientList(new_clients) => {
                    clients.data = new_clients;
                }
                ServerMessage::UpdateBoardPrompts(new_board) => {
                    board.x_size = new_board.x_size;
                    board.y_size = new_board.y_size;
                    board.prompts = new_board.prompts;
                }
                ServerMessage::UpdateBoardActivity(activity) => board.activity = activity.activity,
            },
            Ok(None) => break,
            Err(_) => {
                events.send(DisconnectEvent);
            }
        }
    }
}

fn start_connection(
    mut client: ResMut<Client>,
    mut state: ResMut<NextState<AppState>>,
    mut events: EventReader<ConnectEvent>,
) {
    for event in events.read() {
        state.set(AppState::Playing);
        client
            .open_connection(
                ConnectionConfiguration::from_addrs(
                    event.addr,
                    SocketAddr::from_str("0.0.0.0:0").unwrap(),
                ),
                CertificateVerificationMode::SkipVerification,
            )
            .unwrap();
        client
            .connection()
            .send_message(ClientMessage::Join {
                name: event.username.clone(),
            })
            .unwrap()
    }
}
