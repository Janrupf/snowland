#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(SnowlandControlPanel, snowland_control_panel, SNOWLAND, CONTROL_PANEL,
                     GtkApplication)

/**
 * Creates a new snowland control panel instance.
 *
 * @return the created control panel application instance
 */
SnowlandControlPanel *snowland_control_panel_new();
