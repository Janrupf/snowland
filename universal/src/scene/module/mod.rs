use std::ops::Deref;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

pub use known::*;

use crate::scene::SceneData;

mod clear;
mod known;
mod part;
mod snow;
mod text;

pub trait Module {
    type Config: ModuleConfig;
    type Renderer: ModuleRenderer<Config = Self::Config>;

    /// Creates a renderer.
    fn create_renderer() -> Self::Renderer;

    /// Retrieves the name of the module.
    fn name() -> String;
}

pub trait ModuleConfig: Send + Clone + Default + Serialize + DeserializeOwned {
    /// Renders a menu to change this module configuration.
    fn represent(&mut self, ui: &imgui::Ui);
}

pub trait ModuleRenderer: Send {
    type Config: ModuleConfig;

    /// Renders the module.
    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>);
}

type ModuleWrapperPair = (Box<dyn ModuleContainer>, Box<dyn BoundModuleRenderer>);
type ModuleWrapperCreator = fn() -> ModuleWrapperPair;
type ModuleWrapperDeserializer =
    fn(serde_json::Value) -> Result<ModuleWrapperPair, ModuleConfigError>;

/// Generic wrapper over module types.
#[derive(Debug)]
pub struct ModuleWrapper {
    create: ModuleWrapperCreator,
    deserialize: ModuleWrapperDeserializer,
}

impl ModuleWrapper {
    /// Creates a wrapper instance of the specified module implementation type.
    pub fn of<M>() -> Self
    where
        M: Module + 'static,
    {
        Self {
            create: Self::create_generic::<M>,
            deserialize: Self::deserialize_generic::<M>,
        }
    }

    fn create_generic<M>() -> ModuleWrapperPair
    where
        M: Module + 'static,
    {
        Self::create_generic_with_config::<M>(M::Config::default())
    }

    fn deserialize_generic<M>(
        value: serde_json::Value,
    ) -> Result<ModuleWrapperPair, ModuleConfigError>
    where
        M: Module + 'static,
    {
        let config =
            serde_json::from_value::<M::Config>(value).map_err(ModuleConfigError::Deserialize)?;
        Ok(Self::create_generic_with_config::<M>(config))
    }

    fn create_generic_with_config<M>(config: M::Config) -> ModuleWrapperPair
    where
        M: Module + 'static,
    {
        let shared_config = Arc::new(Mutex::new(config));

        let container = InternalModuleContainer::<M>::new(shared_config.clone());
        let renderer = InternalBoundModuleRenderer::<M>::new(shared_config);

        (Box::new(container), Box::new(renderer))
    }

    /// Creates a new instance of the module with its default configuration.
    pub fn create_with_default_config(&self) -> ModuleWrapperPair {
        (self.create)()
    }

    /// Creates a new instance of the module based on a configuration deserialized from a json value.
    pub fn deserialize_from_config(
        &self,
        config: serde_json::Value,
    ) -> Result<ModuleWrapperPair, ModuleConfigError> {
        (self.deserialize)(config)
    }
}

/// Helper type for configurations shared between a renderer and a container.
type SharedConfig<M>
where
    M: Module,
= Arc<Mutex<M::Config>>;

/// Helper trait to represent this module in the user interface and make it configurable.
pub trait ModuleContainer {
    fn represent(&mut self, ui: &imgui::Ui);

    fn serialize_config(&self) -> Result<serde_json::Value, ModuleConfigError>;

    fn module_type(&self) -> String;
}

struct InternalModuleContainer<M>
where
    M: Module,
{
    config: SharedConfig<M>,
}

impl<M> InternalModuleContainer<M>
where
    M: Module,
{
    pub fn new(config: SharedConfig<M>) -> Self {
        Self { config }
    }
}

impl<M> ModuleContainer for InternalModuleContainer<M>
where
    M: Module,
{
    fn represent(&mut self, ui: &imgui::Ui) {
        let mut config = self.config.lock().expect("Failed to lock ui config");
        config.represent(ui);
    }

    fn serialize_config(&self) -> Result<serde_json::Value, ModuleConfigError> {
        let config = self
            .config
            .lock()
            .expect("Failed to lock serialization config");

        serde_json::to_value(config.deref()).map_err(ModuleConfigError::Serialize)
    }

    fn module_type(&self) -> String {
        M::name()
    }
}

/// Helper trait for a generic renderer bound with a config.
///
/// Instances of this are bound to their config container and scheduled to be removed as soon
/// as the config container is dropped.
pub trait BoundModuleRenderer: Send {
    /// Renders the module to the scene.
    fn render<'a>(&mut self, data: &'a mut SceneData<'a>);

    /// Determines whether this module should be removed because its config container
    /// has been dropped.
    fn should_remove(&self) -> bool;
}

/// Combines a renderer and its associated config into an abstracted type which can
/// be made into a trait object.
struct InternalBoundModuleRenderer<M>
where
    M: Module,
{
    config: SharedConfig<M>,
    renderer: M::Renderer,
}

impl<M> InternalBoundModuleRenderer<M>
where
    M: Module,
{
    /// Creates a new module renderer from the given shared configuration.
    fn new(config: SharedConfig<M>) -> Self {
        Self {
            config,
            renderer: M::create_renderer(),
        }
    }
}

impl<M> BoundModuleRenderer for InternalBoundModuleRenderer<M>
where
    M: Module,
{
    fn render<'a>(&mut self, data: &'a mut SceneData<'a>) {
        let config = self.config.lock().expect("Failed to lock renderer config");
        self.renderer.render(&*config, data)
    }

    fn should_remove(&self) -> bool {
        Arc::strong_count(&self.config) == 1
    }
}

#[derive(Debug, Error)]
pub enum ModuleConfigError {
    #[error("failed to serialize configuration: {0}")]
    Serialize(serde_json::Error),

    #[error("failed to deserialize configuration: {0}")]
    Deserialize(serde_json::Error),
    
    #[error("an I/O error occurred: {0}")]
    Io(#[from] std::io::Error)
}
