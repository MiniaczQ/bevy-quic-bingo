use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

use super::teams::Team;

/// Bingo game mode
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// Only one team can claim a field
    Lockout,
    /// Every team can claim every field
    FFA,
}

impl Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

/// Combined game mode and win condition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoardMode {
    pub game_mode: GameMode,
    pub win_condition: WinCondition,
}

impl Default for BoardMode {
    fn default() -> Self {
        Self {
            game_mode: GameMode::FFA,
            win_condition: WinCondition::InRow { length: 5, rows: 1 },
        }
    }
}

/// Size and prompts of the board
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoardPrompts {
    pub x_size: u8,
    pub y_size: u8,
    pub prompts: Vec<String>,
}

impl Default for BoardPrompts {
    fn default() -> Self {
        Self {
            x_size: 5,
            y_size: 5,
            prompts: vec![String::new(); 25],
        }
    }
}

impl BoardPrompts {
    /// Calculate offset to access a specific field in the prompt vector
    pub fn offset(&self, x: u8, y: u8) -> usize {
        x as usize * self.y_size as usize + y as usize
    }

    /// Get a read-only access to a prompt
    pub fn prompt(&self, x: u8, y: u8) -> &String {
        let offset = self.offset(x, y);
        &self.prompts[offset]
    }
}

/// Activity of all teams on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardActivity {
    pub activity: Vec<HashSet<Team>>,
}

impl BoardActivity {
    /// Instance with no activity
    pub fn empty(size: usize) -> Self {
        Self {
            activity: vec![HashSet::new(); size],
        }
    }
}

/// Game-constant configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoardConfig {
    pub mode: BoardMode,
    pub prompts: BoardPrompts,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            prompts: Default::default(),
        }
    }
}

/// Game-constant and dynamic board data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub config: BoardConfig,
    pub activity: BoardActivity,
}

impl Board {
    pub fn reset_activity(&mut self) {
        self.activity = BoardActivity::empty(self.config.prompts.prompts.len());
    }

    pub fn offset(&self, x: u8, y: u8) -> usize {
        self.config.prompts.offset(x, y)
    }

    pub fn prompt(&self, x: u8, y: u8) -> &String {
        self.config.prompts.prompt(x, y)
    }

    pub fn activity(&self, x: u8, y: u8) -> &HashSet<Team> {
        let offset = self.offset(x, y);
        &self.activity.activity[offset]
    }

    pub fn is_active(&self, x: u8, y: u8, team: &Team) -> bool {
        let offset = self.offset(x, y);
        self.activity.activity[offset].contains(team)
    }

    pub fn activity_mut(&mut self, x: u8, y: u8) -> &mut HashSet<Team> {
        let offset = self.offset(x, y);
        &mut self.activity.activity[offset]
    }

    pub fn check_win(&self) -> Option<Team> {
        match self.config.mode.win_condition {
            WinCondition::InRow { length, rows } => self.check_win_in_row(length, rows),
            WinCondition::Domination => self.check_win_domination(),
            WinCondition::FirstTo(n) => self.check_win_first_to(n),
        }
    }

    fn check_win_in_row(&self, length: u8, rows: u8) -> Option<Team> {
        for team in Team::iter() {
            let mut winning_rows = 0;
            let x_size = self.config.prompts.x_size;
            let y_size = self.config.prompts.y_size;
            // L-R
            if length <= y_size {
                for sx in 0..x_size {
                    'xy: for sy in 0..y_size + 1 - length {
                        for d in 0..length {
                            if !self.is_active(sx, sy + d, team) {
                                continue 'xy;
                            }
                        }
                        winning_rows += 1;
                    }
                }
            }
            // T-D
            if length <= x_size {
                for sx in 0..x_size + 1 - length {
                    'xy: for sy in 0..y_size {
                        for d in 0..length {
                            if !self.is_active(sx + d, sy, team) {
                                continue 'xy;
                            }
                        }
                        winning_rows += 1;
                    }
                }
            }
            // TL-BR
            if length <= x_size && length <= y_size {
                for sx in 0..x_size + 1 - length {
                    'xy: for sy in 0..y_size + 1 - length {
                        for d in 0..length {
                            if !self.is_active(sx + d, sy + d, team) {
                                continue 'xy;
                            }
                        }
                        winning_rows += 1;
                    }
                }
            }
            // BL-TR
            if length <= x_size && length <= y_size {
                for sx in 0..x_size + 1 - length {
                    'xy: for sy in 0..y_size + 1 - length {
                        for d in 0..length {
                            if !self.is_active(sx + d, y_size - sy - d - 1, team) {
                                continue 'xy;
                            }
                        }
                        winning_rows += 1;
                    }
                }
            }

            if winning_rows >= rows {
                return Some(*team);
            }
        }
        None
    }

    fn check_win_first_to(&self, n: u8) -> Option<Team> {
        for team in Team::iter() {
            let mut count = 0;
            for x in 0..self.config.prompts.x_size {
                for y in 0..self.config.prompts.y_size {
                    if self.is_active(x, y, team) {
                        count += 1;
                    }
                }
            }
            if count >= n {
                return Some(*team);
            }
        }
        None
    }

    fn check_win_domination(&self) -> Option<Team> {
        if self.config.mode.game_mode != GameMode::Lockout {
            return None;
        }
        let mut total_count = 0;
        let mut team_counts = Vec::<(Team, u32)>::new();
        for team in Team::iter() {
            let mut count = 0;
            for x in 0..self.config.prompts.x_size {
                for y in 0..self.config.prompts.y_size {
                    if self.is_active(x, y, team) {
                        count += 1;
                    }
                }
            }
            team_counts.push((*team, count));
            total_count += count;
        }
        let free_space =
            self.config.prompts.x_size as u32 * self.config.prompts.y_size as u32 - total_count;
        team_counts.sort_by(|a, b| b.1.cmp(&a.1));
        if free_space + team_counts[1].1 < team_counts[0].1 {
            return Some(team_counts[0].0);
        }
        None
    }
}

impl Default for Board {
    fn default() -> Self {
        let config = BoardConfig::default();
        let activity = BoardActivity::empty(config.prompts.prompts.len());
        Self { config, activity }
    }
}

/// Game win condition
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum WinCondition {
    /// Get N lines of M length
    InRow { length: u8, rows: u8 },
    /// Get most fields
    Domination,
    /// Get at least N fields
    FirstTo(u8),
}

impl Display for WinCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WinCondition::InRow { length, rows } => {
                f.write_fmt(format_args!("{} rows of {}", rows, length))
            }
            WinCondition::Domination => f.write_str("Domination"),
            WinCondition::FirstTo(n) => f.write_fmt(format_args!("First to {}", n)),
        }
    }
}
