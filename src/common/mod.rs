use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use bevy::prelude::*;

use self::bingo::{Board, Mode};

pub mod bingo;
pub mod protocol;
pub mod teams;

#[derive(Resource)]
pub struct BoardRes(pub Board);

impl Default for BoardRes {
    fn default() -> Self {
        Self(Board {
            mode: Mode::FFA,
            x_size: 5,
            y_size: 5,
            prompts: vec![String::from("boop"); 25],
            activity: vec![HashSet::new(); 25],
        })
    }
}

impl Deref for BoardRes {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoardRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
