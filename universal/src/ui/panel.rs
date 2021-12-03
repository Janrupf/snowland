use imgui::{Condition, TableColumnFlags, TableColumnSetup, TableFlags, Ui, Window};

use crate::ui::module_list::ModuleList;
use crate::RendererController;

pub struct MainPanel {
    modules: ModuleList,
}

const NO_MODULE_MESSAGE: &str = concat!(
    "\
Select a module on the left to configure.
Or add a new module by using the drop down.\
",
    "\n\n",
    env!("CARGO_PKG_NAME"),
    " v",
    env!("CARGO_PKG_VERSION"),
);

impl MainPanel {
    pub fn new() -> Self {
        Self {
            modules: ModuleList::new(),
        }
    }

    pub fn run(&mut self, ui: &Ui, controller: &RendererController) {
        Window::new("Snowland Control Panel")
            .title_bar(false)
            .movable(false)
            .position([0.0, 0.0], Condition::Always)
            .size(ui.io().display_size, Condition::Always)
            .resizable(false)
            .build(ui, || {
                if let Some(_tok) = ui.begin_table_with_sizing(
                    "Control Panel Layout",
                    2,
                    TableFlags::BORDERS_INNER_V | TableFlags::NO_HOST_EXTEND_Y,
                    ui.content_region_avail(),
                    0.0,
                ) {
                    ui.table_setup_column_with({
                        let mut setup = TableColumnSetup::new("Modules");
                        setup.flags = TableColumnFlags::NO_RESIZE | TableColumnFlags::WIDTH_FIXED;
                        setup.init_width_or_weight = 200.0;
                        setup
                    });

                    ui.table_setup_column_with({
                        let mut setup = TableColumnSetup::new("Control");
                        setup.flags = TableColumnFlags::WIDTH_STRETCH;
                        setup
                    });

                    ui.table_next_row();
                    ui.table_next_column();

                    self.modules.render(ui, controller);
                    ui.table_next_column();

                    if !self.modules.render_selected_container(ui) {
                        self.draw_help_text(ui);
                    }
                }
            });
    }

    fn draw_help_text(&mut self, ui: &Ui) {
        let [cursor_x, cursor_y] = ui.cursor_pos();
        let [available_width, available_height] = ui.content_region_avail();
        let (heights, texts) = NO_MODULE_MESSAGE
            .split('\n')
            .map(|s| (s, ui.calc_text_size(s)))
            .map(|(s, [w, h])| (h, (s, w)))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let height = heights.iter().sum::<f32>();
        let single_height = heights.iter().fold(
            0.0,
            |current, &next| if next > current { next } else { current },
        );

        let start_y = cursor_y + (available_height / 2.0) - (height / 2.0);
        let half_width = available_width / 2.0;

        for (index, (text, width)) in texts.into_iter().enumerate() {
            let x = cursor_x + (half_width - (width / 2.0));
            let y = start_y + (single_height * (index as f32));

            ui.set_cursor_pos([x, y]);
            ui.text(text);
        }
    }
}
