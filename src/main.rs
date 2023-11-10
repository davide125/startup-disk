use adw::prelude::*;
use adw::Application;
use asahi_bless::BootCandidate;
use asahi_bless::Error;
use gtk::{glib, ApplicationWindow, Box, Label, Orientation, ToggleButton};
use rand::Rng;
use std::env;
use uuid::Uuid;

const APP_ID: &str = "org.gnome.StartupDisk";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

type Result<T> = std::result::Result<T, Error>;

fn generate_random_strings(num_strings: usize, max_length: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();

    (0..num_strings)
        .map(|_| {
            let string_length = rng.gen_range(1..=max_length);
            (0..string_length)
                .map(|_| rng.gen_range(b'a'..=b'z') as char)
                .collect()
        })
        .collect()
}

trait StartupDiskTrait {
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>>;
    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()>;
}

struct AsahiBlessLibrary;
impl StartupDiskTrait for AsahiBlessLibrary {
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        // We need root for this to do anything useful
        sudo::escalate_if_needed().unwrap();
        return asahi_bless::get_boot_candidates();
    }

    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()> {
        // We need root for this to do anything useful
        sudo::escalate_if_needed().unwrap();
        return asahi_bless::set_boot_volume(cand, next);
    }
}

struct MockLibrary;
impl StartupDiskTrait for MockLibrary {
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        let mut cands: Vec<BootCandidate> = Vec::new();
        cands.push(BootCandidate {
            vg_uuid: Uuid::new_v4(),
            vol_names: generate_random_strings(2, 10),
            part_uuid: Uuid::new_v4(),
        });
        cands.push(BootCandidate {
            vg_uuid: Uuid::new_v4(),
            vol_names: generate_random_strings(2, 10),
            part_uuid: Uuid::new_v4(),
        });
        cands.push(BootCandidate {
            vg_uuid: Uuid::new_v4(),
            vol_names: generate_random_strings(2, 10),
            part_uuid: Uuid::new_v4(),
        });
        return Ok(cands);
    }

    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()> {
        println!(
            "Setting boot volume: {} {}",
            cand.vol_names.join(", "),
            next
        );
        return Ok(());
    }
}

enum StartupDiskLibrary {
    AsahiBless(AsahiBlessLibrary),
    Mock(MockLibrary),
}

impl StartupDiskTrait for StartupDiskLibrary {
    fn get_boot_candidates(&self) -> Result<Vec<BootCandidate>> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.get_boot_candidates(),
            StartupDiskLibrary::Mock(lib) => lib.get_boot_candidates(),
        }
    }
    fn set_boot_volume(&self, cand: &BootCandidate, next: bool) -> Result<()> {
        match self {
            StartupDiskLibrary::AsahiBless(lib) => lib.set_boot_volume(cand, next),
            StartupDiskLibrary::Mock(lib) => lib.set_boot_volume(cand, next),
        }
    }
}

fn build_window(app: &Application) -> ApplicationWindow {
    let container = Box::new(Orientation::Vertical, 2);
    let buttons_container = Box::new(Orientation::Horizontal, 10);
    let mut last_button: Option<ToggleButton> = None;

    let use_mock_library = env::var("USE_MOCK_LIBRARY").is_ok();
    // Create an instance of the chosen implementation
    let startup_disk_library: &dyn StartupDiskTrait = if use_mock_library {
        &StartupDiskLibrary::Mock(MockLibrary)
    } else {
        &StartupDiskLibrary::AsahiBless(AsahiBlessLibrary)
    };

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
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Startup Disk")
        .child(&container)
        .build();

    return window;
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
