#include "snowland_control_panel.h"

int main(int argc, char** argv) {
  g_autoptr(SnowlandControlPanel) app = snowland_control_panel_new();
  return g_application_run(G_APPLICATION(app), argc, argv);
}
