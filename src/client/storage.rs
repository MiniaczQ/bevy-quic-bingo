use bevy::{ecs::world::FromWorld, log::warn};
use serde::{de::DeserializeOwned, Serialize};

#[derive(bevy::prelude::Resource)]
pub struct Storage<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    runtime: tokio::runtime::Handle,
    data: Option<T>,
    save_task: Option<tokio::task::JoinHandle<Result<()>>>,
    load_task: Option<tokio::task::JoinHandle<Result<T>>>,
}

impl<T> Storage<T>
where
    T: Default + Clone + Serialize + DeserializeOwned + Send + 'static,
{
    pub fn new(runtime: tokio::runtime::Handle) -> Self {
        Self {
            runtime: runtime,
            data: Default::default(),
            save_task: Default::default(),
            load_task: Default::default(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("{0}")]
    TomlDe(#[from] toml::de::Error),
}

type Result<T> = std::result::Result<T, StorageError>;

impl<T> Storage<T>
where
    T: Default + Clone + Serialize + DeserializeOwned + Send + 'static,
{
    pub fn queue_load(&mut self, path: impl AsRef<std::path::Path> + Send + 'static) {
        if let Some(task) = self.load_task.take() {
            if task.is_finished() {
                self.set_from_load_task();
            } else {
                task.abort();
            }
        }
        self.load_task = Some(self.runtime.spawn(async {
            let string = tokio::fs::read_to_string(path).await?;
            let data = toml::de::from_str(&string)?;
            Ok(data)
        }))
    }

    pub fn queue_save(&mut self, path: impl AsRef<std::path::Path> + Send + 'static) {
        if let Some(task) = self.save_task.take() {
            task.abort();
        }
        let data = self.data.clone().unwrap();
        self.save_task = Some(self.runtime.spawn(async move {
            let string = toml::ser::to_string_pretty(&data)?;
            tokio::fs::write(path, string).await?;
            Ok(())
        }))
    }

    pub fn get(&mut self) -> Option<&mut T> {
        if let Some(task) = self.load_task.as_mut() {
            if task.is_finished() {
                self.set_from_load_task();
            }
        };
        self.data.as_mut()
    }

    fn set_from_load_task(&mut self) {
        let result = self
            .runtime
            .block_on(self.load_task.take().unwrap())
            .unwrap();
        match result {
            Ok(v) => self.data = Some(v),
            Err(e) => {
                if let StorageError::Io(e) = &e {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        self.data = Some(T::default());
                        return;
                    }
                }
                warn!(
                    "Failed to load {} due to error {}",
                    std::any::type_name::<T>(),
                    e
                )
            }
        }
    }
}
