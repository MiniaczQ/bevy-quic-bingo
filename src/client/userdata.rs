use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use bevy::{app::AppExit, prelude::*};
use serde::{Deserialize, Serialize};

pub struct UserdataPlugin;

impl Plugin for UserdataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_userdata)
            .add_systems(PostUpdate, save_userdata);
    }
}

const USERDATA_PATH: &str = "userdata.toml";

fn load_userdata(mut commands: Commands) {
    let mby_file = OpenOptions::new().read(true).open(USERDATA_PATH);
    let userdata = match mby_file {
        Ok(mut file) => {
            let mut userdata_str = String::new();
            file.read_to_string(&mut userdata_str).unwrap();
            toml::de::from_str(&userdata_str).unwrap_or_default()
        }
        Err(_) => Userdata::default(),
    };
    commands.insert_resource(userdata);
}

fn save_userdata(mut events: EventReader<AppExit>, userdata: Res<Userdata>) {
    for _ in events.read() {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(USERDATA_PATH)
            .unwrap();
        let userdata = toml::ser::to_string_pretty::<Userdata>(&userdata).unwrap();
        file.write_all(userdata.as_bytes()).unwrap();
    }
}

#[derive(Resource, Serialize, Deserialize, Default)]
pub struct Userdata {
    pub username: String,
    pub addr: String,
}
