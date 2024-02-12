use std::collections::{HashMap, HashSet};

use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join {
        name: String,
    },
    Disconnect {},
    ChangeTeam(Option<u8>),
    UpdateActivity {
        team: u8,
        x: u8,
        y: u8,
        is_active: bool,
    },
    UpdateBoard(BoardPrompts),
    ResetActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardPrompts {
    pub x_size: u8,
    pub y_size: u8,
    pub prompts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardActivity {
    pub activity: Vec<HashSet<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProps {
    pub is_host: bool,
    pub name: String,
    pub team: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    InitClient(ClientId),
    UpdateClientList(HashMap<ClientId, ClientProps>),
    UpdateBoardPrompts(BoardPrompts),
    UpdateBoardActivity(BoardActivity),
}
