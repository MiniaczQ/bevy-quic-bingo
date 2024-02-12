use std::{thread::sleep, time::Duration};

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, connection::ConnectionConfiguration, Client,
    },
    server::ConnectionEvent,
};
use rand::{distributions::Alphanumeric, Rng};

use crate::{
    common::protocol::{ClientMessage, ServerMessage},
    states::AppState,
    Clients,
};

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Connecting), start_connection)
            .add_systems(Update, (handle_server_messages, handle_client_events))
            .add_systems(PostUpdate, on_app_exit);
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

fn start_connection(mut client: ResMut<Client>) {
    client
        .open_connection(
            ConnectionConfiguration::from_strings("127.0.0.1:6000", "0.0.0.0:0").unwrap(),
            CertificateVerificationMode::SkipVerification,
        )
        .unwrap();
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
