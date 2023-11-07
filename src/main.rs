use asahi_bless::{get_boot_candidates, set_boot_volume};
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Box, Orientation, ToggleButton};

const APP_ID: &str = "org.gnome.StartupDisk";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // We need root for this to do anything useful
    sudo::escalate_if_needed().unwrap();

    let container = Box::new(Orientation::Horizontal, 10);
    let cands = get_boot_candidates().unwrap();
    let mut last_button: Option<ToggleButton> = None;

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
            set_boot_volume(&cand, false).unwrap();
        });

        if let Some(ref last) = last_button {
            button.set_group(Some(last));
        }
        container.append(&button);
        last_button = Some(button);
    }

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Startup Disk")
        .child(&container)
        .build();

    // Present window
    window.present();
}
