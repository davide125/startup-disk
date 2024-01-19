// SPDX-License-Identifier: MIT

mod imp {
    use adw::glib;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use adw::Application;

    use crate::startup_disk::startup_disk_library;
    use crate::window::StartupDiskWindow;

    #[derive(Default)]
    pub struct StartupDiskApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for StartupDiskApplication {
        const NAME: &'static str = "StartupDiskApplication";
        type Type = super::StartupDiskApplication;
        type ParentType = Application;
    }

    impl ObjectImpl for StartupDiskApplication {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_actions();
        }
    }

    impl ApplicationImpl for StartupDiskApplication {
        fn activate(&self) {
            let app = self.obj();

            let startup_disk_library = startup_disk_library();
            let window = if let Some(window) = app.active_window() {
                window
            } else {
                let window = StartupDiskWindow::new(&*app, startup_disk_library.is_supported());
                window.upcast()
            };

            window.present();
        }
    }

    impl GtkApplicationImpl for StartupDiskApplication {}
    impl AdwApplicationImpl for StartupDiskApplication {}
}

use adw::gio::{self, ActionEntry, ActionGroup, ActionMap};
use adw::glib;
use adw::gtk;
use adw::prelude::*;
use adw::{AboutWindow, Application};

use crate::config;

glib::wrapper! {
    pub struct StartupDiskApplication(ObjectSubclass<imp::StartupDiskApplication>)
        @extends Application, gtk::Application, gio::Application,
        @implements ActionGroup, ActionMap;
}

impl StartupDiskApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", config::RESOURCE_BASE)
            .build()
    }

    fn setup_actions(&self) {
        // About window action
        let about_action = ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([about_action]);

        // Keyboard shortcuts
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("window.close", &["<primary>w"]);
    }

    fn show_about(&self) {
        let about_window = AboutWindow::from_appdata(
            &format!("{}/{}.metainfo.xml", config::RESOURCE_BASE, config::APP_ID),
            Some(config::APP_VERSION),
        );
        about_window.set_transient_for(self.active_window().as_ref());
        about_window.set_destroy_with_parent(true);

        about_window.present();
    }
}
