// SPDX-License-Identifier: MIT

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk::{Box, FlowBox, Image, Label, MenuButton, Orientation, ScrolledWindow, ToggleButton};

use crate::config;
use crate::startup_disk::startup_disk_library;

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

        // Connect to "toggled" signal; this fires on every state change
        button.connect_toggled(move |button| {
            /* We only want to set the boot volume for the button that's being
            toggled active (i.e. the one the user clicked, if it wasn't
            active already. */
            if button.is_active() {
                if startup_disk_library.needs_escalation("set_boot_volume") {
                    sudo::escalate_if_needed().unwrap();
                }
                startup_disk_library.set_boot_volume(&cand, false).unwrap();
            }
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

pub fn build_app_window(app: &Application) -> ApplicationWindow {
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
        .title(config::APP_NAME)
        .content(&content)
        .resizable(false)
        .build()
}
