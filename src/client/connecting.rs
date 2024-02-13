use std::{net::SocketAddr, str::FromStr, thread::sleep, time::Duration};

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode,
    connection::{ConnectionConfiguration, ConnectionEvent},
    Client, QuinnetClientPlugin,
};
use rand::{distributions::Alphanumeric, Rng};

use crate::{
    common::protocol::{ClientMessage, ServerMessage},
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
            .add_plugins(QuinnetClientPlugin::default())
            .add_systems(
                Update,
                start_connection.run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(
                Update,
                (handle_server_messages, handle_client_events).run_if(in_state(AppState::Playing)),
            )
            .add_systems(PostUpdate, on_app_exit.run_if(in_state(AppState::Playing)));
    }
}

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, client: Res<Client>) {
    if !app_exit_events.is_empty() {
        client
            .connection()
            .send_message(ClientMessage::Disconnect {})
            .unwrap();
        // TODO Clean: event to let the async client send his last messages.
        sleep(Duration::from_secs_f32(0.1));
    }
}

fn handle_server_messages(
    mut clients: ResMut<Clients>,
    mut client: ResMut<Client>,
    mut state: ResMut<NextState<AppState>>,
) {
    while let Some(message) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::InitClient(self_id) => {
                clients.self_id = self_id;
                info!("Connected with id {}", self_id);
                state.set(AppState::Playing);
            }
            ServerMessage::UpdateClientList(new_clients) => {
                clients.data = new_clients;
            }
            ServerMessage::UpdateBoardPrompts(_) => todo!(),
            ServerMessage::UpdateBoardActivity(_) => todo!(),
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

fn handle_client_events(
    mut connection_events: EventReader<ConnectionEvent>,
    client: ResMut<Client>,
) {
    if !connection_events.is_empty() {
        let username: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        info!("Joining with name: {}", username);

        client
            .connection()
            .send_message(ClientMessage::Join { name: username })
            .unwrap();

        connection_events.clear();
    }
}
