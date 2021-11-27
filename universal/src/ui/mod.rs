use std::fmt::{Debug, Display, Formatter};
use std::sync::mpsc::Receiver;

use iui::controls::{Button, VerticalBox};
use iui::prelude::*;
use iui::UIError;

use crate::ControlMessage;

pub struct SnowlandUI {
    inner: UI,
    window: Window,
    message_receiver: Receiver<ControlMessage>,
    visible: bool,
}

impl SnowlandUI {
    pub fn new() -> Result<Self, Error> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let inner = UI::init()?;
        let mut window = Window::new(&inner, "Snowland", 200, 200, WindowType::NoMenubar);
        window.on_closing(&inner, {
            let inner = inner.clone();

            move |w| {
                w.hide(&inner);
                drop(sender.send(ControlMessage::CloseUI));
            }
        });

        let mut vbox = VerticalBox::new(&inner);

        let mut button = Button::new(&inner, "Click me!");
        button.on_clicked(&inner, {
            let inner = inner.clone();
            move |x| {
                log::debug!("Clicked!");
                x.set_text(&inner, "Clicked!");
            }
        });
        vbox.append(&inner, button, LayoutStrategy::Compact);

        window.set_child(&inner, vbox);

        Ok(Self {
            inner,
            window,
            message_receiver: receiver,
            visible: false,
        })
    }

    pub fn tick(&mut self, incoming_messages: &[ControlMessage]) -> Vec<ControlMessage> {
        for message in incoming_messages {
            match message {
                ControlMessage::OpenUI => {
                    self.window.show(&self.inner);
                    self.visible = true;
                }

                ControlMessage::CloseUI => {
                    self.window.hide(&self.inner);
                    self.visible = false;
                }

                _ => {}
            }
        }

        if self.visible {
            self.inner.event_loop().next_tick(&self.inner);
        }

        let mut messages = Vec::new();

        while let Ok(v) = self.message_receiver.try_recv() {
            messages.push(v);
        }

        messages
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Error(UIError);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl From<UIError> for Error {
    fn from(err: UIError) -> Self {
        Self(err)
    }
}

impl std::error::Error for Error {}
