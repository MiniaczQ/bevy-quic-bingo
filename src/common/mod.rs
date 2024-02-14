use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use self::bingo::{Board, BoardMode};

pub mod bingo;
pub mod protocol;
pub mod teams;

#[derive(Resource, Default)]
pub struct BoardRes(pub Board);

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

#[derive(Resource, Default)]
pub struct BoardModeRes(BoardMode);

impl Deref for BoardModeRes {
    type Target = BoardMode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoardModeRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
