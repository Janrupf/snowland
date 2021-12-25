use crate::rendering::display::Display;

#[derive(Debug)]
pub struct Context<'a> {
    displays: &'a Vec<Display>,
}

impl<'a> Context<'a> {
    pub fn new(displays: &'a Vec<Display>) -> Self {
        Self { displays }
    }

    pub fn displays(&self) -> &'a Vec<Display> {
        self.displays
    }
}
