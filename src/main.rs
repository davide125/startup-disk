// SPDX-License-Identifier: MIT

mod application;
mod config;
mod startup_disk;
mod window;

use adw::prelude::*;
use gtk::{gio, glib};

use application::StartupDiskApplication;

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("startup-disk.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = StartupDiskApplication::new();

    // Run the application
    app.run()
}
