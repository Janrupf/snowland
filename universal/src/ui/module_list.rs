use std::ops::{Add, Sub};

use imgui::{ChildWindow, InputText, ItemHoveredFlags, MouseButton, Selectable, Ui};

use crate::scene::module::{BoundModuleRenderer, KnownModules, ModuleContainer, ModuleWrapper};
use crate::RendererController;

/// Sidebar controller for inserted modules.
pub struct ModuleList {
    entries: Vec<ModuleEntry>,
    add_types: Vec<(&'static String, &'static ModuleWrapper)>,
    selected_add_type: usize,
    next_id: i32,
    selected_module: Option<usize>,
    dragging_item: Option<(usize, usize)>,
}

impl ModuleList {
    /// Creates a new module list and initializes its empty state.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            add_types: KnownModules::iter().collect(),
            selected_add_type: 0,
            next_id: 0,
            selected_module: None,
            dragging_item: None,
        }
    }

    /// Renders the list into the UI and processes user input.
    pub fn render(&mut self, ui: &Ui, controller: &RendererController) {
        ChildWindow::new("Module Column")
            .border(false)
            .always_auto_resize(false)
            .build(ui, || {
                ui.combo(
                    "###Type",
                    &mut self.selected_add_type,
                    &self.add_types,
                    |(name, _)| (*name).into(),
                );

                ui.same_line();

                if ui.button("Add") {
                    self.add_module(controller);
                }

                ui.separator();

                ChildWindow::new("Module List").border(false).build(ui, || {
                    let list_start = ui.cursor_screen_pos()[1];

                    for i in 0..self.entries.len() {
                        let state = self.entries[i].render_sidebar(ui);

                        match state {
                            ModuleEntryState::NoModify => {
                                if ui.is_item_active() && !ui.is_item_hovered() {
                                    self.dragging_item = Some((i, i));
                                }
                            }
                            ModuleEntryState::Remove => {
                                self.entries.remove(i);

                                match self.selected_module {
                                    Some(x) if x == i => {
                                        self.selected_module = None;
                                    }
                                    Some(x) if x > i => {
                                        self.selected_module = Some(x - 1);
                                    }
                                    _ => {}
                                }
                                break;
                            }
                            ModuleEntryState::Select => {
                                self.selected_module = Some(i);
                            }
                        }
                    }

                    let cursor_y_diff = ui.cursor_screen_pos()[1] - list_start;

                    if self.entries.is_empty() || !ui.is_mouse_dragging(MouseButton::Left) {
                        self.dragging_item = None;
                    }

                    if let Some((original_position, current_position)) = self.dragging_item {
                        let mouse_y = ui.io().mouse_pos[1];
                        let item_height = cursor_y_diff / self.entries.len() as f32;

                        let new_position = {
                            let relative_y = mouse_y - list_start;

                            (relative_y / item_height) as i32
                        }
                        .clamp(0, (self.entries.len() - 1) as _)
                            as usize;

                        self.entries.swap(current_position, new_position);
                        self.dragging_item = Some((original_position, new_position));
                        controller.swap_modules(current_position, new_position);

                        if Some(current_position) == self.selected_module {
                            self.selected_module = Some(new_position);
                        }
                    }
                });
            });
    }

    /// Renders the currently selected container, if any.
    pub fn render_selected_container(&mut self, ui: &Ui) -> bool {
        match self.selected_module {
            Some(i) => {
                self.entries[i].render_container(ui);
                true
            }

            None => false,
        }
    }

    /// Helper function to add a module to the list.
    fn add_module(&mut self, controller: &RendererController) {
        let (name, wrapper) = self.add_types[self.selected_add_type];

        let (entry, renderer) = ModuleEntry::new(self.next_id, name.clone(), wrapper);

        self.entries.push(entry);
        self.next_id += 1;

        controller.insert_module(renderer);
    }
}

impl Default for ModuleList {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the current state of a module entry.
#[derive(Eq, PartialEq)]
enum ModuleEntryState {
    /// The entry should be kept as-is.
    NoModify,

    /// The entry should be removed.
    Remove,

    /// The entry should be marked as the selected one.
    Select,
}

/// Control interface for modules.
struct ModuleEntry {
    id: i32,
    name: String,
    container: Box<dyn ModuleContainer>,
}

impl ModuleEntry {
    /// Creates a new module entry and its underlying module.
    pub fn new(
        id: i32,
        name: String,
        wrapper: &ModuleWrapper,
    ) -> (Self, Box<dyn BoundModuleRenderer>) {
        let (container, renderer) = wrapper.create_with_default_config();

        (
            Self {
                id,
                name,
                container,
            },
            renderer,
        )
    }

    /// Renders the sidebar content of the module.
    pub fn render_sidebar(&mut self, ui: &Ui) -> ModuleEntryState {
        let _id = ui.push_id(self.id);
        let remove = ui.button("-");

        ui.same_line();
        let selected = Selectable::new(&self.name).build(ui);

        match (remove, selected) {
            (true, _) => ModuleEntryState::Remove,
            (_, true) => ModuleEntryState::Select,
            _ => ModuleEntryState::NoModify,
        }
    }

    /// Renders the internal module UI.
    pub fn render_container(&mut self, ui: &Ui) {
        InputText::new(ui, "Name", &mut self.name)
            .hint("Module name")
            .allow_tab_input(false)
            .build();

        self.container.represent(ui);
    }
}
