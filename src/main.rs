// SPDX-License-Identifier: MIT

mod application;
mod boot_candidate;
mod config;
mod privileged;
mod startup_disk;
mod window;

use adw::prelude::*;
use gtk::{gio, glib};

use application::StartupDiskApplication;

fn main() -> glib::ExitCode {
    if let Some(exit_code) = privileged::maybe_handle_privileged_invocation() {
        return exit_code;
    }

    // Register and include resources
    gio::resources_register_include!("startup-disk.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = StartupDiskApplication::new();

    // Run the application
    app.run()
}
