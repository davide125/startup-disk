// SPDX-License-Identifier: MIT

mod imp {
    use adw::glib;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use asahi_bless::BootCandidate;
    use std::cell::RefCell;

    #[derive(glib::Properties, Default)]
    #[properties(wrapper_type = super::BootCandidateObject)]
    pub struct BootCandidateObject {
        #[property(get, set)]
        name: RefCell<String>,

        pub boot_candidate: RefCell<Option<BootCandidate>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BootCandidateObject {
        const NAME: &'static str = "StartupDiskBootCandidateObject";
        type Type = super::BootCandidateObject;
    }

    impl ObjectImpl for BootCandidateObject {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

use adw::glib::{self, subclass::types::ObjectSubclassIsExt};
use asahi_bless::BootCandidate;

glib::wrapper! {
    pub struct BootCandidateObject(ObjectSubclass<imp::BootCandidateObject>);
}

impl BootCandidateObject {
    pub fn new(candidate: BootCandidate) -> Self {
        let object: BootCandidateObject = glib::Object::builder()
            .property("name", &candidate.vol_names[1])
            .build();
        *object.imp().boot_candidate.borrow_mut() = Some(candidate);

        object
    }
}
