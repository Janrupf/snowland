#include "snowland_control_panel.h"

#include <flutter_linux/flutter_linux.h>

#ifdef GDK_WINDOWING_X11

#include <gdk/gdkx.h>

#endif

#include "flutter/generated_plugin_registrant.h"

struct _SnowlandControlPanel {
    GtkApplication parent_instance;
    char **dart_entrypoint_arguments;
};

G_DEFINE_TYPE(SnowlandControlPanel, snowland_control_panel, GTK_TYPE_APPLICATION)

static gboolean on_window_delete(GtkWidget *window, GdkEvent *event, gpointer data) {
    printf("Sending shutdown to event channel!\n");

    g_autoptr(FlEventChannel) channel = FL_EVENT_CHANNEL(data);
    g_autoptr(FlValue) value = fl_value_new_string("shutdown");

    fl_event_channel_send(
            channel,
            value,
            nullptr,
            nullptr
    );

    fl_event_channel_send_end_of_stream(channel, nullptr, nullptr);

    return FALSE;
}

// Implements GApplication::activate.
static void snowland_control_panel_activate(GApplication *application) {
    SnowlandControlPanel *self = SNOWLAND_CONTROL_PANEL(application);
    GtkWindow *window =
            GTK_WINDOW(gtk_application_window_new(GTK_APPLICATION(application)));

    // Use a header bar when running in GNOME as this is the common style used
    // by applications and is the setup most users will be using (e.g. Ubuntu
    // desktop).
    // If running on X and not using GNOME then just use a traditional title bar
    // in case the window manager does more exotic layout, e.g. tiling.
    // If running on Wayland assume the header bar will work (may need changing
    // if future cases occur).
    gboolean use_header_bar = TRUE;
#ifdef GDK_WINDOWING_X11
    GdkScreen *screen = gtk_window_get_screen(window);
    if (GDK_IS_X11_SCREEN(screen)) {
        const gchar *wm_name = gdk_x11_screen_get_window_manager_name(screen);
        if (g_strcmp0(wm_name, "GNOME Shell") != 0) {
            use_header_bar = FALSE;
        }
    }
#endif
    if (use_header_bar) {
        GtkHeaderBar *header_bar = GTK_HEADER_BAR(gtk_header_bar_new());
        gtk_widget_show(GTK_WIDGET(header_bar));
        gtk_header_bar_set_title(header_bar, "Snowland control panel");
        gtk_header_bar_set_show_close_button(header_bar, TRUE);
        gtk_window_set_titlebar(window, GTK_WIDGET(header_bar));
    } else {
        gtk_window_set_title(window, "Snowland control panel");
    }

    g_autoptr(FlDartProject) project = fl_dart_project_new();
    fl_dart_project_set_dart_entrypoint_arguments(project, self->dart_entrypoint_arguments);
    FlView *view = fl_view_new(project);

    FlEngine *engine = fl_view_get_engine(view);
    g_autoptr(FlBinaryMessenger) messenger = fl_engine_get_binary_messenger(engine);
    g_autoptr(FlStandardMethodCodec) codec = fl_standard_method_codec_new();
    FlEventChannel *platform_channel = fl_event_channel_new(
            messenger,
            "native_platform_events",
            FL_METHOD_CODEC(codec)
    );

    gtk_window_set_default_size(window, 1280, 720);
    g_signal_connect(G_OBJECT(window), "delete-event", G_CALLBACK(on_window_delete), platform_channel);

    gtk_widget_show(GTK_WIDGET(window));

    gtk_widget_show(GTK_WIDGET(view));
    gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(view));

    fl_register_plugins(FL_PLUGIN_REGISTRY(view));

    gtk_widget_grab_focus(GTK_WIDGET(view));
}

// Implements GApplication::local_command_line.
static gboolean
snowland_control_panel_local_command_line(GApplication *application, gchar ***arguments, int *exit_status) {
    SnowlandControlPanel *self = SNOWLAND_CONTROL_PANEL(application);
    // Strip out the first argument as it is the binary name.
    self->dart_entrypoint_arguments = g_strdupv(*arguments + 1);

    g_autoptr(GError) error = nullptr;
    if (!g_application_register(application, nullptr, &error)) {
        g_warning("Failed to register: %s", error->message);
        *exit_status = 1;
        return TRUE;
    }

    g_application_activate(application);
    *exit_status = 0;

    return TRUE;
}

// Implements GObject::dispose.
static void snowland_control_panel_dispose(GObject *object) {
    SnowlandControlPanel *self = SNOWLAND_CONTROL_PANEL(object);
    g_clear_pointer(&self->dart_entrypoint_arguments, g_strfreev);
    G_OBJECT_CLASS(snowland_control_panel_parent_class)->dispose(object);
}

static void snowland_control_panel_class_init(SnowlandControlPanelClass *klass) {
    G_APPLICATION_CLASS(klass)->activate = snowland_control_panel_activate;
    G_APPLICATION_CLASS(klass)->local_command_line = snowland_control_panel_local_command_line;
    G_OBJECT_CLASS(klass)->dispose = snowland_control_panel_dispose;
}

static void snowland_control_panel_init(SnowlandControlPanel *self) {}

SnowlandControlPanel *snowland_control_panel_new() {
    return SNOWLAND_CONTROL_PANEL(g_object_new(snowland_control_panel_get_type(),
                                               "application-id", APPLICATION_ID,
                                               "flags", G_APPLICATION_NON_UNIQUE,
                                               nullptr));
}
