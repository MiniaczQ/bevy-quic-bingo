use std::collections::HashSet;

use bevy::prelude::*;

use self::bingo::Board;

pub mod bingo;
pub mod protocol;

#[derive(Resource)]
pub struct BoardRes(pub Board);

impl Default for BoardRes {
    fn default() -> Self {
        Self(Board {
            x_size: 5,
            y_size: 5,
            prompts: vec![String::new(); 25],
            activity: vec![HashSet::new(); 25],
        })
    }
}
