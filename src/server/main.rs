#[path = "../common/mod.rs"]
mod common;

use std::collections::{HashMap, HashSet};

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
    protocol::{BoardActivity, BoardPrompts, ClientMessage, ClientProps, ServerMessage},
    BoardRes,
};

#[derive(Resource, Debug, Clone, Default)]
struct Clients {
    data: HashMap<ClientId, ClientProps>,
}

fn broadcast(endpoint: &Endpoint, clients: &Clients, msg: ServerMessage) {
    endpoint
        .send_group_message(clients.data.keys().into_iter(), msg)
        .unwrap();
}

fn broadcast_clients(endpoint: &Endpoint, clients: &Clients) {
    broadcast(
        endpoint,
        clients,
        ServerMessage::UpdateClientList(clients.data.clone()),
    );
}

fn broadcast_board_prompts(endpoint: &Endpoint, clients: &Clients, prompts: BoardPrompts) {
    broadcast(
        endpoint,
        clients,
        ServerMessage::UpdateBoardPrompts(prompts),
    );
}

fn broadcast_board_activity(endpoint: &Endpoint, clients: &Clients, activity: BoardActivity) {
    broadcast(
        endpoint,
        clients,
        ServerMessage::UpdateBoardActivity(activity),
    );
}

fn handle_client_messages(
    mut server: ResMut<Server>,
    mut clients: ResMut<Clients>,
    mut board: ResMut<BoardRes>,
) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some(message) = endpoint.try_receive_message_from::<ClientMessage>(client_id) {
            match message {
                ClientMessage::Join { name: username } => {
                    if clients.data.contains_key(&client_id) {
                        warn!(
                            "Received a Join from an already connected client: {}",
                            client_id
                        );
                        endpoint.disconnect_client(client_id).unwrap();
                    } else {
                        info!("{} connected", username);
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
                        broadcast_clients(endpoint, &clients);
                    }
                }
                ClientMessage::Disconnect {} => {
                    endpoint.disconnect_client(client_id).unwrap();
                    handle_disconnect(endpoint, &mut clients, client_id);
                }
                ClientMessage::ChangeTeam(new_team) => {
                    let client = clients.data.get_mut(&client_id).unwrap();
                    client.team = new_team;
                    info!("{} changed team to {:?}", client.username, client.team);
                    broadcast_clients(endpoint, &clients);
                }
                ClientMessage::UpdateActivity {
                    team,
                    x,
                    y,
                    is_active,
                } => {
                    let client = clients.data.get_mut(&client_id).unwrap();
                    info!(
                        "{} changed activity for team {:?} at {}, {} to {}",
                        client.username, team, x, y, is_active
                    );
                    let activity = board.activity_mut(x, y);
                    match !is_active {
                        true => activity.remove(&team),
                        false => activity.insert(team),
                    };

                    broadcast_board_activity(
                        &endpoint,
                        &clients,
                        BoardActivity {
                            activity: board.0.activity.clone(),
                        },
                    )
                }
                ClientMessage::UpdateBoard(new_board) => {
                    let client = clients.data.get_mut(&client_id).unwrap();
                    if !client.is_host {
                        continue;
                    }
                    let flat_size = new_board.prompts.len();
                    board.0 = Board {
                        mode: new_board.mode,
                        x_size: new_board.x_size,
                        y_size: new_board.y_size,
                        prompts: new_board.prompts.clone(),
                        activity: vec![HashSet::default(); flat_size],
                    };
                    broadcast_board_prompts(&endpoint, &clients, new_board);
                }
                ClientMessage::ResetActivity => {
                    let client = clients.data.get_mut(&client_id).unwrap();
                    if !client.is_host {
                        continue;
                    }
                    let flat_size = board.0.prompts.len();
                    board.0.activity = vec![HashSet::default(); flat_size];
                    broadcast_board_activity(
                        &endpoint,
                        &clients,
                        BoardActivity {
                            activity: board.0.activity.clone(),
                        },
                    )
                }
            }
        }
    }
}

fn handle_server_events(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut server: ResMut<Server>,
    mut clients: ResMut<Clients>,
) {
    // The server signals us about users that lost connection
    for client in connection_lost_events.read() {
        handle_disconnect(server.endpoint_mut(), &mut clients, client.id);
    }
}

fn handle_disconnect(endpoint: &mut Endpoint, clients: &mut ResMut<Clients>, client_id: ClientId) {
    if let Some(client) = clients.data.remove(&client_id) {
        // Host migration
        if client.is_host {
            if let Some(client) = clients.data.iter_mut().next() {
                client.1.is_host = true;
            }
        }
        broadcast_clients(endpoint, &clients);
        info!("{} disconnected", client.username);
    } else {
        warn!(
            "Received a Disconnect from an unknown or disconnected client: {}",
            client_id
        )
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
        .add_systems(Update, (handle_client_messages, handle_server_events))
        .run();
}
