use std::ops::Deref;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub use known::*;

use crate::scene::SceneData;

mod clear;
mod countdown;
mod image;
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

pub trait ModuleConfig: Send + Clone + Default + Serialize + DeserializeOwned {}

pub trait ModuleRenderer: Send {
    type Config: ModuleConfig;

    /// Renders the module.
    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>);
}

type ModuleWrapperCreator = fn() -> Box<dyn ModuleContainer>;
type ModuleWrapperDeserializer =
    fn(serde_json::Value) -> Result<Box<dyn ModuleContainer>, ModuleConfigError>;

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

    fn create_generic<M>() -> Box<dyn ModuleContainer>
    where
        M: Module + 'static,
    {
        Self::create_generic_with_config::<M>(M::Config::default())
    }

    fn deserialize_generic<M>(
        value: serde_json::Value,
    ) -> Result<Box<dyn ModuleContainer>, ModuleConfigError>
    where
        M: Module + 'static,
    {
        let config =
            serde_json::from_value::<M::Config>(value).map_err(ModuleConfigError::Deserialize)?;
        Ok(Self::create_generic_with_config::<M>(config))
    }

    fn create_generic_with_config<M>(config: M::Config) -> Box<dyn ModuleContainer>
    where
        M: Module + 'static,
    {
        let container = InternalModuleContainer::<M>::new(M::create_renderer(), config);
        Box::new(container)
    }

    /// Creates a new instance of the module with its default configuration.
    pub fn create_with_default_config(&self) -> Box<dyn ModuleContainer> {
        (self.create)()
    }

    /// Creates a new instance of the module based on a configuration deserialized from a json value.
    pub fn deserialize_from_config(
        &self,
        config: serde_json::Value,
    ) -> Result<Box<dyn ModuleContainer>, ModuleConfigError> {
        (self.deserialize)(config)
    }
}

pub trait ModuleContainer {
    fn serialize_config(&self) -> Result<serde_json::Value, ModuleConfigError>;

    fn update_config(&mut self, new_config: serde_json::Value) -> Result<(), ModuleConfigError>;

    fn module_type(&self) -> String;

    fn run_frame<'a>(&mut self, data: &mut SceneData<'a>);
}

struct InternalModuleContainer<M>
where
    M: Module,
{
    renderer: M::Renderer,
    config: M::Config,
}

impl<M> InternalModuleContainer<M>
where
    M: Module,
{
    pub fn new(renderer: M::Renderer, config: M::Config) -> Self {
        Self { renderer, config }
    }
}

impl<M> ModuleContainer for InternalModuleContainer<M>
where
    M: Module,
{
    fn serialize_config(&self) -> Result<serde_json::Value, ModuleConfigError> {
        serde_json::to_value(&self.config).map_err(ModuleConfigError::Serialize)
    }

    fn update_config(&mut self, new_config: Value) -> Result<(), ModuleConfigError> {
        self.config = serde_json::from_value(new_config).map_err(ModuleConfigError::Deserialize)?;
        Ok(())
    }

    fn module_type(&self) -> String {
        M::name()
    }

    fn run_frame<'a>(&mut self, data: &mut SceneData<'a>) {
        self.renderer.render(&self.config, data)
    }
}

#[derive(Debug, Error)]
pub enum ModuleConfigError {
    #[error("failed to serialize configuration: {0}")]
    Serialize(serde_json::Error),

    #[error("failed to deserialize configuration: {0}")]
    Deserialize(serde_json::Error),

    #[error("an I/O error occurred: {0}")]
    Io(#[from] std::io::Error),
}
