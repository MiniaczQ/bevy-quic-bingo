use std::collections::HashMap;

use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

use super::{
    bingo::{BoardActivity, BoardMode, BoardPrompts},
    teams::Team,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join {
        name: String,
    },
    Disconnect {},
    ChangeTeam(Option<Team>),
    UpdateActivity {
        team: Team,
        x: u8,
        y: u8,
        is_active: bool,
    },
    SetPrompts(BoardPrompts),
    SetMode(BoardMode),
    Kick(ClientId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProps {
    pub is_host: bool,
    pub username: String,
    pub team: Option<Team>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    InitClient(ClientId),
    SetClients(HashMap<ClientId, ClientProps>),
    SetMode(BoardMode),
    SetPrompts(BoardPrompts),
    SetActivity(BoardActivity),
}
