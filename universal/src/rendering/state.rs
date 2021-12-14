use std::sync::mpsc::{Receiver, Sender};

use crate::io::ConfigIO;
use crate::scene::module::{BoundModuleRenderer, ModuleContainer, ModuleWrapperPair};

/// Messages which can be sent to the renderer.
pub enum RendererStateMessage {
    /// Signals the renderer to shut down.
    Shutdown,

    /// Inserts a renderer module.
    InsertModule {
        module: Box<dyn BoundModuleRenderer>,
    },

    /// Swaps the position of 2 modules.
    Swap(usize, usize),
}

pub struct RendererController {
    sender: Sender<RendererStateMessage>,
}

impl RendererController {
    /// Creates a new renderer controller and splits it into the controller and
    /// receiver.
    pub fn new() -> (Self, Receiver<RendererStateMessage>) {
        let (sender, receiver) = std::sync::mpsc::channel();

        (Self { sender }, receiver)
    }

    /// Inserts a module into the renderer at the end.
    pub fn insert_module(&self, module: Box<dyn BoundModuleRenderer>) {
        drop(
            self.sender
                .send(RendererStateMessage::InsertModule { module }),
        )
    }

    /// Swaps the position of 2 renderer modules.
    pub fn swap_modules(&self, a: usize, b: usize) {
        drop(self.sender.send(RendererStateMessage::Swap(a, b)))
    }

    /// Sends the renderer the shutdown signal.
    pub fn shutdown(self) {
        drop(self.sender.send(RendererStateMessage::Shutdown));
    }

    /// Loads the modules from the configuration.
    pub fn load(&self) -> Vec<ModuleWrapperPair> {
        log::info!("Loading modules...");
        match ConfigIO::load() {
            Ok(v) => {
                log::info!("Loaded {} modules successfully!", v.len());
                v
            }
            Err(err) => {
                log::error!("Failed to load modules: {}", err);
                Vec::new()
            }
        }
    }

    /// Saves the modules into the configuration.
    pub fn save<'a>(&self, modules: impl Iterator<Item = &'a Box<dyn ModuleContainer>>) {
        log::info!("Saving modules...");
        if let Err(err) = ConfigIO::save(modules) {
            log::error!("Failed to save modules: {}", err);
        } else {
            log::info!("Modules saved successfully!");
        }
    }
}
