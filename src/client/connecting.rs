use std::{collections::HashSet, net::SocketAddr, str::FromStr, thread::sleep, time::Duration};

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode,
    connection::{ConnectionConfiguration, ConnectionLostEvent},
    Client, QuinnetClientPlugin,
};

use crate::{
    common::{
        bingo::Board,
        protocol::{ClientMessage, ServerMessage},
        BoardRes,
    },
    states::AppState,
    Clients,
};

#[derive(Event)]
pub struct StartConnection {
    pub username: String,
    pub addr: SocketAddr,
}

#[derive(Event)]
pub struct StopConnection;

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartConnection>()
            .add_event::<StopConnection>()
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
                stop_connection.run_if(in_state(AppState::Playing)),
            );
    }
}

fn start_connection(
    mut client: ResMut<Client>,
    mut state: ResMut<NextState<AppState>>,
    mut events: EventReader<StartConnection>,
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

pub fn stop_connection(
    mut client: ResMut<Client>,
    mut stop_connection: EventReader<StopConnection>,
    mut connection_lost: EventReader<ConnectionLostEvent>,
    mut app_exit: EventReader<AppExit>,
    mut state: ResMut<NextState<AppState>>,
) {
    if app_exit.is_empty() && connection_lost.is_empty() && stop_connection.is_empty() {
        return;
    }

    app_exit.clear();
    connection_lost.clear();
    stop_connection.clear();

    client
        .get_connection()
        .map(|c| c.try_send_message(ClientMessage::Disconnect {}));
    sleep(Duration::from_secs_f32(0.1));
    client.close_all_connections().ok();
    state.set(AppState::MainMenu);
}

fn handle_server_messages(
    mut clients: ResMut<Clients>,
    mut client: ResMut<Client>,
    mut board: ResMut<BoardRes>,
    mut events: EventWriter<StopConnection>,
) {
    loop {
        let result = client.connection_mut().receive_message::<ServerMessage>();
        match result {
            Ok(Some(msg)) => handle_single_message(&mut board, &mut clients, msg),
            Ok(None) => break,
            Err(_) => {
                events.send(StopConnection);
                break;
            }
        }
    }
}

fn handle_single_message(board: &mut Board, clients: &mut Clients, msg: ServerMessage) {
    match msg {
        ServerMessage::InitClient(self_id) => {
            clients.self_id = self_id;
            info!("Connected with id {}", self_id);
        }
        ServerMessage::UpdateClientList(new_clients) => {
            clients.data = new_clients;
        }
        ServerMessage::UpdateBoardPrompts(new_board) => {
            board.mode = new_board.mode;
            board.win_condition = new_board.win_condition;
            board.x_size = new_board.x_size;
            board.y_size = new_board.y_size;
            board.prompts = new_board.prompts;
            board.activity = vec![HashSet::default(); board.prompts.len()]
        }
        ServerMessage::UpdateBoardActivity(activity) => board.activity = activity.activity,
    }
}
