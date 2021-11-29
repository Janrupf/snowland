use std::sync::mpsc::{Receiver, Sender};

use crate::scene::module::BoundModuleRenderer;

/// Messages which can be sent to the renderer.
pub enum RendererStateMessage {
    /// Signals the renderer to shut down.
    Shutdown,

    /// Inserts a renderer module.
    InsertModule {
        index: usize,
        module: Box<dyn BoundModuleRenderer>,
    },
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

    /// Inserts a module into the renderer at the specified position.
    pub fn insert_module(&self, index: usize, module: Box<dyn BoundModuleRenderer>) {
        drop(
            self.sender
                .send(RendererStateMessage::InsertModule { index, module }),
        )
    }

    /// Sends the renderer the shutdown signal.
    pub fn shutdown(self) {
        drop(self.sender.send(RendererStateMessage::Shutdown));
    }
}
