use std::collections::{HashMap, HashSet};

use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

use super::{
    bingo::{Board, Mode, WinCondition},
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
    UpdateBoard(BoardPrompts),
    Kick(ClientId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardPrompts {
    pub mode: Mode,
    pub win_condition: WinCondition,
    pub x_size: u8,
    pub y_size: u8,
    pub prompts: Vec<String>,
}

impl BoardPrompts {
    pub fn from_board(board: &Board) -> Self {
        Self {
            mode: board.mode,
            win_condition: board.win_condition,
            x_size: board.x_size,
            y_size: board.y_size,
            prompts: board.prompts.clone(),
        }
    }

    pub fn same_as_board(&self, board: &Board) -> bool {
        self.mode == board.mode
            && self.win_condition == board.win_condition
            && self.x_size == board.x_size
            && self.y_size == board.y_size
            && self.prompts == board.prompts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardActivity {
    pub activity: Vec<HashSet<Team>>,
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
    UpdateClientList(HashMap<ClientId, ClientProps>),
    UpdateBoardPrompts(BoardPrompts),
    UpdateBoardActivity(BoardActivity),
}
