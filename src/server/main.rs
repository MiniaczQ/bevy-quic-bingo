use std::collections::HashMap;

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfiguration,
    },
    shared::ClientId,
};

use common::{
    bingo::Board,
    protocol::{ClientMessage, ClientProps, ServerMessage},
    BoardRes,
};

use common::bingo::GameMode;

#[derive(Resource, Debug, Clone, Default)]
struct Clients {
    data: HashMap<ClientId, ClientProps>,
}

fn broadcast(endpoint: &Endpoint, clients: &Clients, msg: ServerMessage) {
    endpoint.try_send_group_message(clients.data.keys(), msg);
}

fn handle_messages(
    mut server: ResMut<Server>,
    mut clients: ResMut<Clients>,
    mut board: ResMut<BoardRes>,
) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some(message) = endpoint.try_receive_message_from::<ClientMessage>(client_id) {
            handle_single_message(&mut board, endpoint, &mut clients, message, client_id);
        }
    }
}

fn handle_single_message(
    board: &mut Board,
    endpoint: &mut Endpoint,
    clients: &mut Clients,
    message: ClientMessage,
    client_id: ClientId,
) {
    match message {
        ClientMessage::Join { name: username } => {
            if clients.data.contains_key(&client_id) {
                return;
            }

            if clients.data.values().any(|x| x.username == username) {
                endpoint.disconnect_client(client_id).unwrap();
                return;
            }

            let is_host = clients.data.is_empty();
            clients.data.insert(
                client_id,
                ClientProps {
                    is_host,
                    username: username.clone(),
                    team: None,
                },
            );
            endpoint
                .send_message(client_id, ServerMessage::InitClient(client_id))
                .unwrap();
            endpoint
                .send_message(client_id, ServerMessage::SetMode(board.config.mode.clone()))
                .unwrap();
            endpoint
                .send_message(
                    client_id,
                    ServerMessage::SetPrompts(board.config.prompts.clone()),
                )
                .unwrap();
            endpoint
                .send_message(
                    client_id,
                    ServerMessage::SetActivity(board.activity.clone()),
                )
                .unwrap();
            broadcast(
                endpoint,
                clients,
                ServerMessage::SetClients(clients.data.clone()),
            );
        }
        ClientMessage::Disconnect {} => {
            endpoint.disconnect_client(client_id).unwrap();
            handle_disconnect(endpoint, clients, client_id);
        }
        ClientMessage::ChangeTeam(new_team) => {
            let client = clients.data.get_mut(&client_id).unwrap();
            client.team = new_team;
            broadcast(
                endpoint,
                clients,
                ServerMessage::SetClients(clients.data.clone()),
            );
        }
        ClientMessage::UpdateActivity {
            team,
            x,
            y,
            is_active,
        } => {
            let mode = board.config.mode.game_mode;
            let win = board.check_win();
            let activity = board.activity_mut(x, y);
            match is_active {
                true => {
                    if (mode != GameMode::Lockout || activity.is_empty()) && win.is_none() {
                        activity.insert(team);
                    }
                }
                false => {
                    activity.remove(&team);
                }
            };

            broadcast(
                endpoint,
                clients,
                ServerMessage::SetActivity(board.activity.clone()),
            );
        }
        ClientMessage::SetMode(mode) => {
            let client = clients.data.get_mut(&client_id).unwrap();
            if !client.is_host {
                return;
            }
            board.config.mode = mode.clone();
            board.reset_activity();
            broadcast(endpoint, clients, ServerMessage::SetMode(mode));
        }
        ClientMessage::SetPrompts(prompts) => {
            let client = clients.data.get_mut(&client_id).unwrap();
            if !client.is_host {
                return;
            }
            board.config.prompts = prompts.clone();
            board.reset_activity();
            broadcast(endpoint, clients, ServerMessage::SetPrompts(prompts));
        }
        ClientMessage::ResetActivity => {
            let client = clients.data.get_mut(&client_id).unwrap();
            if !client.is_host {
                return;
            }
            board.reset_activity();
            broadcast(
                endpoint,
                clients,
                ServerMessage::SetActivity(board.activity.clone()),
            );
        }
        ClientMessage::Kick(client_id) => {
            endpoint.try_disconnect_client(client_id);
            handle_disconnect(endpoint, clients, client_id);
        }
    }
}

fn handle_connection_lost(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut server: ResMut<Server>,
    mut clients: ResMut<Clients>,
) {
    for client in connection_lost_events.read() {
        handle_disconnect(server.endpoint_mut(), &mut clients, client.id);
    }
}

fn handle_disconnect(endpoint: &mut Endpoint, clients: &mut Clients, client_id: ClientId) {
    if let Some(client) = clients.data.remove(&client_id) {
        if client.is_host {
            if let Some(client) = clients.data.iter_mut().next() {
                client.1.is_host = true;
            }
        }
        broadcast(
            endpoint,
            clients,
            ServerMessage::SetClients(clients.data.clone()),
        );
    }
}

fn start_listening(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfiguration::from_string("0.0.0.0:6000").unwrap(),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "127.0.0.1".to_string(),
            },
        )
        .unwrap();
}

fn main() {
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::default(),
            LogPlugin::default(),
            QuinnetServerPlugin::default(),
        ))
        .insert_resource(Clients::default())
        .insert_resource(BoardRes::default())
        .add_systems(Startup, start_listening)
        .add_systems(Update, (handle_messages, handle_connection_lost))
        .run();
}
