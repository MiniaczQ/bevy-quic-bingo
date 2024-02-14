use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use self::bingo::{Board, BoardMode, BoardPrompts};

pub mod bingo;
pub mod protocol;
pub mod teams;

#[derive(Resource, Default)]
pub struct BoardRes {
    pub board: Board,
    pub changed: bool,
}

impl Deref for BoardRes {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}

impl DerefMut for BoardRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.board
    }
}

#[derive(Resource, Default)]
pub struct ConfMode(BoardMode);

impl Deref for ConfMode {
    type Target = BoardMode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ConfMode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource, Default)]
pub struct ConfPrompts {
    pub prompts: BoardPrompts,
    pub changed: bool,
}

impl Deref for ConfPrompts {
    type Target = BoardPrompts;

    fn deref(&self) -> &Self::Target {
        &self.prompts
    }
}

impl DerefMut for ConfPrompts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prompts
    }
}
