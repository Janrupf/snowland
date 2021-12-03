mod known;
mod text;

pub use known::*;

use std::sync::{Arc, Mutex};

use crate::scene::SceneData;

pub trait Module {
    type Config: ModuleConfig;
    type Renderer: ModuleRenderer<Config = Self::Config>;

    /// Creates a renderer.
    fn create_renderer() -> Self::Renderer;

    /// Retrieves the name of the module.
    fn name() -> String;
}

pub trait ModuleConfig: Send + Clone + Default {
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

/// Generic wrapper over module types.
#[derive(Debug)]
pub struct ModuleWrapper {
    create: ModuleWrapperCreator,
}

impl ModuleWrapper {
    /// Creates a wrapper instance of the specified module implementation type.
    pub fn of<M>() -> Self
    where
        M: Module + 'static,
    {
        Self {
            create: Self::create_generic::<M>,
        }
    }

    fn create_generic<M>() -> ModuleWrapperPair
    where
        M: Module + 'static,
    {
        let shared_config = Arc::new(Mutex::new(M::Config::default()));

        let container = InternalModuleContainer::<M>::new(shared_config.clone());
        let renderer = InternalBoundModuleRenderer::<M>::new(shared_config);

        (Box::new(container), Box::new(renderer))
    }

    /// Creates a new instance of the module with its default configuration.
    pub fn create_with_default_config(&self) -> ModuleWrapperPair {
        (self.create)()
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
