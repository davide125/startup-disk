use adw::prelude::*;
use adw::Application;
use gtk::{gio, glib, ApplicationWindow, Box, Label, Orientation, ToggleButton};
use startup_disk::startup_disk_library;

const APP_ID: &str = "org.gnome.StartupDisk";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Hook up actions
    app.set_accels_for_action("app.quit", &["<primary>q"]);
    let quit_action = gio::ActionEntry::builder("quit")
        .activate(|app: &Application, _, _| app.quit())
        .build();
    app.add_action_entries([quit_action]);

    // Run the application
    app.run()
}

fn build_window(app: &Application) -> ApplicationWindow {
    let container = Box::new(Orientation::Vertical, 2);
    let buttons_container = Box::new(Orientation::Horizontal, 10);
    let mut last_button: Option<ToggleButton> = None;

    let startup_disk_library = startup_disk_library();

    let cands = startup_disk_library.get_boot_candidates().unwrap();

    for cand in cands {
        let button = ToggleButton::builder()
            .label(cand.vol_names.join(", "))
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // Connect to "clicked" signal of `button`
        button.connect_clicked(move |_button| {
            startup_disk_library.set_boot_volume(&cand, false).unwrap();
        });

        if let Some(ref last) = last_button {
            button.set_group(Some(last));
        }
        buttons_container.append(&button);
        last_button = Some(button);
    }

    let label = Label::builder()
        .label("Select the disk you want to use to start up from")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&label);
    container.append(&buttons_container);

    // Create a window
    ApplicationWindow::builder()
        .application(app)
        .title("Startup Disk")
        .child(&container)
        .build()
}

fn build_ui(app: &Application) {
    // Get the current window or create one if necessary
    let window = if let Some(window) = app.active_window() {
        window
    } else {
        let window = build_window(app);
        window.upcast()
    };

    // Present the window
    window.present();
}
