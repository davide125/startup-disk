// SPDX-License-Identifier: MIT

mod imp {
    use adw::glib;
    use adw::gtk;
    use adw::subclass::prelude::*;
    use adw::ApplicationWindow;
    use gtk::glib::subclass::InitializingObject;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/org/startup-disk/StartupDisk/gtk/window.ui")]
    pub struct StartupDiskWindow;

    #[glib::object_subclass]
    impl ObjectSubclass for StartupDiskWindow {
        const NAME: &'static str = "StartupDiskWindow";
        type Type = super::StartupDiskWindow;
        type ParentType = ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StartupDiskWindow {}
    impl WidgetImpl for StartupDiskWindow {}
    impl WindowImpl for StartupDiskWindow {}
    impl ApplicationWindowImpl for StartupDiskWindow {}
    impl AdwApplicationWindowImpl for StartupDiskWindow {}
}

use adw::gio::{ActionGroup, ActionMap};
use adw::glib::{wrapper, IsA, Object};
use adw::gtk::{
    Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager, Widget, Window,
};
use adw::{gtk, Application, ApplicationWindow};

wrapper! {
    pub struct StartupDiskWindow(ObjectSubclass<imp::StartupDiskWindow>)
        @extends ApplicationWindow, gtk::ApplicationWindow, Window, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl StartupDiskWindow {
    pub fn new<A: IsA<Application>>(application: &A) -> Self {
        Object::builder()
            .property("application", application)
            .build()
    }
}
