use bevy::ecs::world::FromWorld;
use serde::{de::DeserializeOwned, Serialize};

pub struct Storage<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    runtime: tokio::runtime::Handle,
    data: Option<T>,
    save_task: Option<tokio::task::JoinHandle<Result<(), toml::ser::Error>>>,
    load_task: Option<tokio::task::JoinHandle<Result<T, toml::de::Error>>>,
}

impl<T> FromWorld for Storage<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let runtime = world.resource::<bevy_quinnet::shared::AsyncRuntime>();
        Self {
            runtime: runtime.handle().clone(),
            data: Default::default(),
            save_task: Default::default(),
            load_task: Default::default(),
        }
    }
}

impl<T> Storage<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    pub fn load(&mut self) {
        if let Some(task) = self.load_task.take() {
            task.abort();
        }
    }

    pub fn save(&mut self) {
        if let Some(task) = self.load_task.take() {
            task.abort();
        }
    }

    pub fn get(&mut self) -> Option<&mut T> {
        if let Some(task) = self.load_task.as_mut() {
            if task.is_finished() {
                let result = self.runtime.block_on(task);
            }
        };
        self.data.as_mut()
    }
}
