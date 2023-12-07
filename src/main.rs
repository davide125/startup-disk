// SPDX-License-Identifier: MIT

mod config;
mod startup_disk;
mod window;

use adw::prelude::*;
use adw::{AboutWindow, Application, ApplicationWindow, MessageDialog, ResponseAppearance};
use gtk::{gio, glib};

use startup_disk::startup_disk_library;

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("startup-disk.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder()
        .application_id(config::APP_ID)
        .resource_base_path(config::RESOURCE_BASE)
        .build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Hook up actions
    let about_action = gio::ActionEntry::builder("about")
        .activate(|app: &Application, _, _| show_about(app))
        .build();
    let quit_action = gio::ActionEntry::builder("quit")
        .activate(|app: &Application, _, _| app.quit())
        .build();
    app.add_action_entries([about_action, quit_action]);
    app.set_accels_for_action("app.quit", &["<primary>q"]);
    app.set_accels_for_action("window.close", &["<primary>w"]);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let startup_disk_library = startup_disk_library();
    if startup_disk_library.is_supported() {
        // Get the current window or create one if necessary
        let window = if let Some(window) = app.active_window() {
            window
        } else {
            let window = window::build_app_window(app);
            window.upcast()
        };

        // Present the window
        window.present();
    } else {
        let window = ApplicationWindow::builder()
            .application(app)
            .title(config::APP_NAME)
            .build();
        let dialog = MessageDialog::builder()
            .heading(config::APP_NAME)
            .body("Startup Disk is only supported on Apple Silicon Macs")
            .transient_for(&window)
            .modal(true)
            .destroy_with_parent(true)
            .build();
        dialog.add_responses(&[("ok", "Ok")]);
        dialog.set_response_appearance("ok", ResponseAppearance::Suggested);
        dialog.connect_response(None, move |_dialog, _response| {
            window.destroy();
        });
        dialog.present();
    }
}

fn show_about(app: &Application) {
    let window = AboutWindow::from_appdata(
        &format!("{}/{}.metainfo.xml", config::RESOURCE_BASE, config::APP_ID),
        Some(config::APP_VERSION),
    );
    window.set_transient_for(app.active_window().as_ref());
    window.set_destroy_with_parent(true);

    window.present();
}
