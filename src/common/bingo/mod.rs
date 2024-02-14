use std::collections::HashSet;

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

    pub fn activity_mut(&mut self, x: u8, y: u8) -> &mut HashSet<Team> {
        let offset = self.offset(x, y);
        &mut self.activity[offset]
    }
}
