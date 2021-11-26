use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread::JoinHandle;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::SendNotifyMessageA;

use snowland_universal::control::ControlMessage;

use crate::shell::integration::WM_SNOWLAND_MESSENGER;
use crate::shell::Error;

#[derive(Debug)]
pub enum ReceiveResult<T> {
    Message(T),
    None,
    Shutdown,
}

#[derive(Debug)]
pub enum HostToIntegrationMessage {
    QuitLoop,
    Control(ControlMessage),
}

#[derive(Debug)]
pub enum InternalIntegrationToHostMessage {
    WindowCreated(HWND),
    Message(ControlMessage),
}

#[derive(Debug)]
pub struct HostMessenger {
    joiner: Option<JoinHandle<Result<(), Error>>>,
    window: Option<HWND>,
    receiver: Receiver<InternalIntegrationToHostMessage>,
}

impl HostMessenger {
    pub fn new(
        joiner: JoinHandle<Result<(), Error>>,
        receiver: Receiver<InternalIntegrationToHostMessage>,
    ) -> Self {
        Self {
            joiner: Some(joiner),
            window: None,
            receiver,
        }
    }

    pub fn receive(&mut self) -> ReceiveResult<ControlMessage> {
        match self.receiver.try_recv() {
            Ok(InternalIntegrationToHostMessage::WindowCreated(window)) => {
                self.window.replace(window);
                ReceiveResult::None
            }
            Ok(InternalIntegrationToHostMessage::Message(v)) => ReceiveResult::Message(v),
            Err(err) => match err {
                TryRecvError::Disconnected => ReceiveResult::Shutdown,
                TryRecvError::Empty => ReceiveResult::None,
            },
        }
    }

    pub fn send(&self, message: HostToIntegrationMessage) {
        let message = Box::into_raw(Box::new(message));

        unsafe {
            SendNotifyMessageA(
                self.window
                    .expect("Tried to send message before window was created"),
                WM_SNOWLAND_MESSENGER,
                WPARAM(message as _),
                LPARAM(0),
            );
        }
    }
}

impl Drop for HostMessenger {
    fn drop(&mut self) {
        if let Err(err) = self
            .joiner
            .take()
            .unwrap()
            .join()
            .expect("Integration panicked")
        {
            log::error!("Shell integration finished with error: {}", err)
        }
    }
}

#[derive(Debug)]
pub struct IntegrationMessenger {
    sender: Sender<InternalIntegrationToHostMessage>,
}

impl IntegrationMessenger {
    pub fn new(sender: Sender<InternalIntegrationToHostMessage>) -> Self {
        Self { sender }
    }

    pub fn window_created(&self, window: HWND) {
        drop(
            self.sender
                .send(InternalIntegrationToHostMessage::WindowCreated(window)),
        );
    }

    pub fn send(&self, message: ControlMessage) {
        drop(
            self.sender
                .send(InternalIntegrationToHostMessage::Message(message)),
        );
    }
}
