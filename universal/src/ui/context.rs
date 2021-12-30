use crate::rendering::display::Display;

#[derive(Debug)]
pub struct Context<'a> {
    displays: &'a [Display],
}

impl<'a> Context<'a> {
    pub fn new(displays: &'a [Display]) -> Self {
        Self { displays }
    }

    pub fn displays(&self) -> &'a [Display] {
        self.displays
    }
}
