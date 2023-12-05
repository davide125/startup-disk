// SPDX-License-Identifier: MIT

use adw::prelude::*;
use adw::{
    AboutWindow, Application, ApplicationWindow, HeaderBar, MessageDialog, ResponseAppearance,
};
use const_format::concatcp;
use gtk::{
    gio, glib, Box, FlowBox, Image, Label, MenuButton, Orientation, ScrolledWindow, ToggleButton,
};
use startup_disk::startup_disk_library;

const APP_NAME: &str = "Startup Disk";
const APP_ID: &str = "org.gnome.StartupDisk";
const APP_VERSION: &str = "0.1.0";
const RESOURCE_BASE: &str = "/org/gnome/startup-disk";

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("startup-disk.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder()
        .application_id(APP_ID)
        .resource_base_path(RESOURCE_BASE)
        .build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Hook up actions
    let about_action = gio::ActionEntry::builder("about")
        .activate(|_, _, _| show_about())
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

fn build_boot_candidates() -> ScrolledWindow {
    let buttons_container = FlowBox::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .row_spacing(12)
        .column_spacing(12)
        .build();
    let mut last_button: Option<ToggleButton> = None;

    let startup_disk_library = startup_disk_library();
    if startup_disk_library.needs_escalation("get_boot_volume") {
        sudo::escalate_if_needed().unwrap();
    }
    let default_cand = startup_disk_library.get_boot_volume(false).unwrap();
    if startup_disk_library.needs_escalation("get_boot_candidates") {
        sudo::escalate_if_needed().unwrap();
    }
    let cands = startup_disk_library.get_boot_candidates().unwrap();

    for cand in cands {
        let is_default: bool =
            (cand.part_uuid == default_cand.part_uuid) && (cand.vg_uuid == default_cand.vg_uuid);

        let button_content = Box::new(Orientation::Vertical, 0);
        button_content.append(
            &Image::builder()
                .icon_name("drive-harddisk")
                .pixel_size(128)
                .build(),
        );
        button_content.append(&Label::builder().label(&cand.vol_names[1]).build());

        let button = ToggleButton::builder()
            .child(&button_content)
            .active(is_default)
            .build();

        // Connect to "clicked" signal of `button`
        button.connect_clicked(move |_button| {
            if startup_disk_library.needs_escalation("set_boot_volume") {
                sudo::escalate_if_needed().unwrap();
            }
            startup_disk_library.set_boot_volume(&cand, false).unwrap();
        });

        if let Some(ref last) = last_button {
            button.set_group(Some(last));
        }
        buttons_container.append(&button);
        last_button = Some(button);
    }

    ScrolledWindow::builder()
        .child(&buttons_container)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .build()
}

fn build_app_window(app: &Application) -> ApplicationWindow {
    let content = Box::new(Orientation::Vertical, 0);

    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&app.menubar().unwrap())
        .build();

    let header_bar = HeaderBar::new();
    header_bar.pack_end(&menu_button);

    let label = Label::builder()
        .label("Select the disk you want to use to start up from")
        .margin_top(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    content.append(&header_bar);
    content.append(&label);
    content.append(&build_boot_candidates());

    // Create a window
    ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .content(&content)
        .resizable(false)
        .build()
}

fn build_ui(app: &Application) {
    let startup_disk_library = startup_disk_library();
    if startup_disk_library.is_supported() {
        // Get the current window or create one if necessary
        let window = if let Some(window) = app.active_window() {
            window
        } else {
            let window = build_app_window(app);
            window.upcast()
        };

        // Present the window
        window.present();
    } else {
        let window = ApplicationWindow::builder()
            .application(app)
            .title(APP_NAME)
            .build();
        let dialog = MessageDialog::builder()
            .heading(APP_NAME)
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

fn show_about() {
    let window = AboutWindow::from_appdata(
        concatcp!(RESOURCE_BASE, "/", APP_ID, ".metainfo.xml"),
        Some(APP_VERSION),
    );

    window.present();
}
