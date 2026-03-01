// SPDX-License-Identifier: MIT

mod imp {
    use adw::gio::ListStore;
    use adw::glib::{self, subclass::InitializingObject};
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use adw::{
        gtk::{GridView, Stack},
        ApplicationWindow, StatusPage,
    };
    use std::cell::RefCell;

    #[derive(gtk::CompositeTemplate, glib::Properties, Default)]
    #[template(resource = "/org/startup-disk/StartupDisk/window.ui")]
    #[properties(wrapper_type = super::StartupDiskWindow)]
    pub struct StartupDiskWindow {
        #[template_child]
        pub stack: TemplateChild<Stack>,
        #[template_child]
        pub grid_view: TemplateChild<GridView>,
        #[template_child]
        pub error_status_page: TemplateChild<StatusPage>,

        pub boot_candidates: RefCell<Option<ListStore>>,

        #[property(get, set)]
        supported: RefCell<bool>,
    }

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

    impl ObjectImpl for StartupDiskWindow {
        fn constructed(&self) {
            self.parent_constructed();

            // Setup grid
            self.obj().setup_list_store();
            self.obj().setup_factory();

            // Add signal for supported property
            self.obj().connect_notify(Some("supported"), |window, _| {
                if window.supported() {
                    match window.add_boot_candidates() {
                        Ok(()) => {
                            window.imp().stack.set_visible_child_name("boot_candidates");
                        }
                        Err(e) => {
                            window
                                .imp()
                                .error_status_page
                                .set_description(Some(&e.to_string()));
                            window.imp().stack.set_visible_child_name("error");
                        }
                    }
                }
            });
        }

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

    impl WidgetImpl for StartupDiskWindow {}
    impl WindowImpl for StartupDiskWindow {}
    impl ApplicationWindowImpl for StartupDiskWindow {}
    impl AdwApplicationWindowImpl for StartupDiskWindow {}
}

use adw::gio::{ActionGroup, ActionMap, ListStore};
use adw::glib::{self, subclass::types::ObjectSubclassIsExt};
use adw::prelude::*;
use adw::{
    gtk::{
        Accessible, Buildable, ConstraintTarget, ListItem, Native, Root, ShortcutManager, Widget,
        Window,
    },
    Application, ApplicationWindow,
};

use crate::boot_candidate::object::BootCandidateObject;
use crate::boot_candidate::BootCandidateWidget;
use crate::startup_disk::{startup_disk_library, StartupDiskError};

glib::wrapper! {
    pub struct StartupDiskWindow(ObjectSubclass<imp::StartupDiskWindow>)
        @extends ApplicationWindow, gtk::ApplicationWindow, Window, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl StartupDiskWindow {
    pub fn new<A: IsA<Application>>(application: &A, supported: bool) -> Self {
        glib::Object::builder()
            .property("application", application)
            .property("supported", supported)
            .build()
    }

    /// Convenience function to borrow and clone the list store
    fn get_list_store(&self) -> ListStore {
        self.imp().boot_candidates.borrow().clone().unwrap()
    }

    /// Creates the list store and sets up a single selection model
    fn setup_list_store(&self) {
        let list_store = ListStore::new::<BootCandidateObject>();
        self.imp().boot_candidates.replace(Some(list_store));

        let selection_model = adw::gtk::SingleSelection::new(Some(self.get_list_store()));
        let window = self.clone();
        selection_model.connect_selection_changed(move |selection, _, _| {
            if let Some(object) = selection
                .selected_item()
                .and_downcast::<BootCandidateObject>()
            {
                let startup_disk_library = startup_disk_library();
                if let Err(e) = startup_disk_library.set_boot_volume(
                    "/dev/mtd/by-name/nvram",
                    object.imp().boot_candidate.borrow().as_ref().unwrap(),
                    false,
                ) {
                    let dialog = adw::AlertDialog::builder()
                        .heading("Could Not Set Boot Volume")
                        .body(e.to_string())
                        .build();
                    dialog.add_response("ok", "OK");
                    dialog.present(Some(&window));
                }
            }
        });
        self.imp().grid_view.set_model(Some(&selection_model));
    }

    /// Creates the factory which creates, binds, and unbinds boot candidate widgets
    fn setup_factory(&self) {
        let factory = adw::gtk::SignalListItemFactory::new();

        // Creates widgets
        factory.connect_setup(|_, list_item| {
            let widget = BootCandidateWidget::new();
            list_item.set_property("child", Some(&widget));
        });

        // Binds widget properties to object properties
        factory.connect_bind(|_, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let object = list_item
                .item()
                .and_downcast::<BootCandidateObject>()
                .unwrap();
            let widget = list_item
                .child()
                .and_downcast::<BootCandidateWidget>()
                .unwrap();

            widget.bind(&object);
        });

        // Unbinds widget properties from object properties
        factory.connect_unbind(|_, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let widget = list_item
                .child()
                .and_downcast::<BootCandidateWidget>()
                .unwrap();
            widget.unbind();
        });

        self.imp().grid_view.set_factory(Some(&factory));
    }

    fn add_boot_candidates(&self) -> Result<(), StartupDiskError> {
        let startup_disk_library = startup_disk_library();

        // Get default boot candidate
        let default_cand = startup_disk_library
            .get_boot_volume("/dev/mtd/by-name/nvram", false)?;

        // Add boot candidates to list store
        for (idx, cand) in startup_disk_library
            .get_boot_candidates()?
            .into_iter()
            .enumerate()
        {
            let is_default =
                cand.part_uuid == default_cand.part_uuid && cand.vg_uuid == default_cand.vg_uuid;

            let object = BootCandidateObject::new(cand);
            self.get_list_store().append(&object);

            if is_default {
                self.imp()
                    .grid_view
                    .model()
                    .unwrap()
                    .select_item(idx as u32, true);
            }
        }
        Ok(())
    }
}
