use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

use super::teams::Team;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Lockout,
    FFA,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub mode: Mode,
    pub win_condition: WinCondition,
    pub x_size: u8,
    pub y_size: u8,
    pub prompts: Vec<String>,
    pub activity: Vec<HashSet<Team>>,
}

impl Board {
    fn offset(&self, x: u8, y: u8) -> usize {
        x as usize * self.y_size as usize + y as usize
    }

    pub fn prompt(&self, x: u8, y: u8) -> &String {
        let offset = self.offset(x, y);
        &self.prompts[offset]
    }

    pub fn prompt_mut(&mut self, x: u8, y: u8) -> &mut String {
        let offset = self.offset(x, y);
        &mut self.prompts[offset]
    }

    pub fn activity(&self, x: u8, y: u8) -> &HashSet<Team> {
        let offset = self.offset(x, y);
        &self.activity[offset]
    }

    pub fn is_active(&self, x: u8, y: u8, team: &Team) -> bool {
        let offset = self.offset(x, y);
        self.activity[offset].contains(team)
    }

    pub fn activity_mut(&mut self, x: u8, y: u8) -> &mut HashSet<Team> {
        let offset = self.offset(x, y);
        &mut self.activity[offset]
    }

    pub fn check_win(&self) -> Option<Team> {
        match self.win_condition {
            WinCondition::InRow { length, rows } => {
                for team in Team::iter() {
                    let mut winning_rows = 0;
                    // L-R
                    for sx in 0..self.x_size {
                        'xy: for sy in 0..self.y_size - length + 1 {
                            for d in 0..length {
                                if !self.is_active(sx, sy + d, team) {
                                    continue 'xy;
                                }
                            }
                            winning_rows += 1;
                        }
                    }
                    // T-D
                    for sx in 0..self.x_size - length + 1 {
                        'xy: for sy in 0..self.y_size {
                            for d in 0..length {
                                if !self.is_active(sx + d, sy, team) {
                                    continue 'xy;
                                }
                            }
                            winning_rows += 1;
                        }
                    }
                    // TL-BR
                    for sx in 0..self.x_size - length + 1 {
                        'xy: for sy in 0..self.y_size - length + 1 {
                            for d in 0..length {
                                if !self.is_active(sx + d, sy + d, team) {
                                    continue 'xy;
                                }
                            }
                            winning_rows += 1;
                        }
                    }
                    // BL-TR
                    for sx in 0..self.x_size - length + 1 {
                        'xy: for sy in 0..self.y_size - length + 1 {
                            for d in 0..length {
                                if !self.is_active(sx + d, self.y_size - sy - d - 1, team) {
                                    continue 'xy;
                                }
                            }
                            winning_rows += 1;
                        }
                    }

                    if winning_rows >= rows {
                        return Some(*team);
                    }
                }
                None
            }
            WinCondition::Domination => {
                if self.mode != Mode::Lockout {
                    return None;
                }

                let mut total_count = 0;
                let mut team_counts = Vec::<(Team, u32)>::new();
                for team in Team::iter() {
                    let mut count = 0;
                    for x in 0..self.x_size {
                        for y in 0..self.y_size {
                            if self.is_active(x, y, team) {
                                count += 1;
                            }
                        }
                    }
                    team_counts.push((*team, count));
                    total_count += count;
                }
                let free_space = self.x_size as u32 * self.y_size as u32 - total_count;

                team_counts.sort_by(|a, b| b.1.cmp(&a.1));
                if free_space + team_counts[1].1 < team_counts[0].1 {
                    return Some(team_counts[0].0);
                }

                None
            }
            WinCondition::FirstTo(n) => {
                for team in Team::iter() {
                    let mut count = 0;
                    for x in 0..self.x_size {
                        for y in 0..self.y_size {
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum WinCondition {
    InRow { length: u8, rows: u8 },
    Domination,
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
