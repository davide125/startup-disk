// SPDX-License-Identifier: MIT

pub mod object;

mod imp {
    use adw::glib::{self, subclass::InitializingObject, Binding};
    use adw::gtk::{self, CompositeTemplate, Label};
    use adw::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/startup-disk/StartupDisk/boot_candidate.ui")]
    pub struct BootCandidateWidget {
        #[template_child]
        pub name: TemplateChild<Label>,

        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BootCandidateWidget {
        const NAME: &'static str = "StartupDiskBootCandidate";
        type Type = super::BootCandidateWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BootCandidateWidget {}
    impl WidgetImpl for BootCandidateWidget {}
    impl BoxImpl for BootCandidateWidget {}
}

use adw::glib;
use adw::gtk::{Accessible, Box, Buildable, ConstraintTarget, Orientable, Widget};
use adw::prelude::*;
use adw::subclass::prelude::*;

use self::object::BootCandidateObject;

glib::wrapper! {
    pub struct BootCandidateWidget(ObjectSubclass<imp::BootCandidateWidget>)
        @extends Box, Widget,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl BootCandidateWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    // Creates bindings to object
    pub fn bind(&self, object: &BootCandidateObject) {
        // Borrow bindings vector
        let mut bindings = self.imp().bindings.borrow_mut();

        // Create binding for the name label
        let name_label = self.imp().name.get();
        let name_binding = object
            .bind_property("name", &name_label, "label")
            .sync_create()
            .build();
        bindings.push(name_binding);
    }

    // Removes bindings
    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}
