use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use self::bingo::Board;

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
