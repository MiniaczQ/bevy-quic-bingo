use std::{net::SocketAddr, str::FromStr, thread::sleep, time::Duration};

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
        teams::Team,
        BoardRes, ConfMode, ConfPrompts,
    },
    fit_text::PromptLayoutCache,
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
            .add_event::<TeamWon>()
            .add_plugins(QuinnetClientPlugin::default())
            .add_systems(
                Update,
                start_connection.run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(Update, handle_messages.run_if(in_state(AppState::Playing)))
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

#[derive(Event)]
pub struct TeamWon(Team);

fn handle_messages(
    mut team_won: EventWriter<TeamWon>,
    mut clients: ResMut<Clients>,
    mut client: ResMut<Client>,
    mut board: ResMut<BoardRes>,
    mut mode_conf: ResMut<ConfMode>,
    mut prompts_conf: ResMut<ConfPrompts>,
    mut events: EventWriter<StopConnection>,
    mut cache: ResMut<PromptLayoutCache>,
) {
    loop {
        let result = client.connection_mut().receive_message::<ServerMessage>();
        match result {
            Ok(Some(msg)) => handle_single_message(
                &mut team_won,
                &mut board,
                &mut clients,
                &mut mode_conf,
                &mut prompts_conf,
                &mut cache,
                msg,
            ),
            Ok(None) => break,
            Err(_) => {
                events.send(StopConnection);
                break;
            }
        }
    }
}

fn handle_single_message(
    team_won: &mut EventWriter<TeamWon>,
    board: &mut Board,
    clients: &mut Clients,
    mode_conf: &mut ConfMode,
    prompts_conf: &mut ConfPrompts,
    cache: &mut PromptLayoutCache,
    msg: ServerMessage,
) {
    match msg {
        ServerMessage::InitClient(self_id) => {
            clients.self_id = self_id;
        }
        ServerMessage::SetClients(new_clients) => {
            clients.data = new_clients;
        }
        ServerMessage::SetMode(mode) => {
            board.config.mode = mode.clone();
            mode_conf.mode = mode;
            mode_conf.changed = false;
            board.reset_activity();
        }
        ServerMessage::SetPrompts(prompts) => {
            board.config.prompts = prompts.clone();
            prompts_conf.prompts = prompts;
            prompts_conf.changed = false;
            board.reset_activity();
            cache.clear();
        }
        ServerMessage::SetActivity(activity) => {
            board.activity = activity;
            if let Some(team) = board.check_win() {
                team_won.send(TeamWon(team));
            }
        }
    }
}
