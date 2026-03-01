// SPDX-License-Identifier: MIT

mod imp {
    use adw::gio::ListStore;
    use adw::glib::{self, subclass::InitializingObject};
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use adw::{
        gtk::{Button, GridView, Stack},
        ApplicationWindow, StatusPage,
    };
    use std::cell::{Cell, RefCell};

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
        #[template_child]
        pub retry_button: TemplateChild<Button>,

        pub boot_candidates: RefCell<Option<ListStore>>,
        pub confirmed_selection: Cell<u32>,
        pub changing_selection: Cell<bool>,

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

            self.obj().connect_notify(Some("supported"), |window, _| {
                if window.supported() {
                    window.try_load_boot_candidates();
                } else {
                    window.show_error(
                        "Startup Disk is only supported on Apple Silicon Macs",
                        false,
                    );
                    window.present();
                }
            });

            // Retry button
            let window_weak = self.obj().downgrade();
            self.obj().imp().retry_button.connect_clicked(move |_| {
                if let Some(window) = window_weak.upgrade() {
                    window.try_load_boot_candidates();
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

use std::io::Cursor;

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

    fn show_error(&self, description: &str, show_retry: bool) {
        self.imp()
            .error_status_page
            .set_description(Some(description));
        self.imp().retry_button.set_visible(show_retry);
        self.imp().stack.set_visible_child_name("error");
    }

    fn try_load_boot_candidates(&self) {
        // Clear any previous candidates
        self.get_list_store().remove_all();

        /* This is neede to keep the window hidden in between the privilege
        escalation prompts */
        self.set_visible(false);

        // We use a callback to populate the window...
        let window = self.clone();
        glib::idle_add_local_once(move || {
            match window.add_boot_candidates() {
                Ok(()) => {
                    window.imp().stack.set_visible_child_name("boot_candidates");
                }
                Err(e) => {
                    window.show_error(&e.to_string(), true);
                }
            }

            // ...and nest another one to resize and make it visible
            let w = window.clone();
            glib::idle_add_local_once(move || {
                w.set_default_size(-1, -1);
                w.present();
            });
        });
    }

    fn set_boot_volume_with_retry(window: &Self, position: u32, object: &BootCandidateObject) {
        let startup_disk_library = startup_disk_library();
        if let Err(e) = startup_disk_library.set_boot_volume(
            "/dev/mtd/by-name/nvram",
            object.imp().boot_candidate.borrow().as_ref().unwrap(),
            false,
        ) {
            let dialog = adw::AlertDialog::builder()
                .heading("Failed to set the boot volume")
                .body(e.to_string())
                .build();
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("retry", "Retry");
            dialog.set_response_appearance("retry", adw::ResponseAppearance::Suggested);
            let window_weak = window.downgrade();
            let object_weak = object.downgrade();
            dialog.choose(window, None::<&adw::gio::Cancellable>, move |response| {
                if let (Some(window), Some(object)) = (window_weak.upgrade(), object_weak.upgrade())
                {
                    if response == "retry" {
                        Self::set_boot_volume_with_retry(&window, position, &object);
                    }
                }
            });
        } else {
            // Only update the selection after successfully setting the boot volume
            window.imp().changing_selection.set(true);
            window
                .imp()
                .grid_view
                .model()
                .unwrap()
                .select_item(position, true);
            window.imp().changing_selection.set(false);
            window.imp().confirmed_selection.set(position);
        }
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
        selection_model.set_autoselect(false);

        let window = self.clone();
        selection_model.connect_selection_changed(move |selection, _, _| {
            if window.imp().changing_selection.get() {
                return;
            }

            let position = selection.selected();
            if let Some(object) = selection
                .selected_item()
                .and_downcast::<BootCandidateObject>()
            {
                // Immediately revert the highlight to the confirmed selection
                let confirmed = window.imp().confirmed_selection.get();
                window.imp().changing_selection.set(true);
                selection.select_item(confirmed, true);
                window.imp().changing_selection.set(false);

                Self::set_boot_volume_with_retry(&window, position, &object);
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
        let default_cand = startup_disk_library.get_boot_volume("/dev/mtd/by-name/nvram", false)?;

        // Load volume icons (best-effort)
        let icons = startup_disk_library.get_volume_icons().unwrap_or_default();

        // Add boot candidates to list store
        for (idx, cand) in startup_disk_library
            .get_boot_candidates()?
            .into_iter()
            .enumerate()
        {
            let is_default =
                cand.part_uuid == default_cand.part_uuid && cand.vg_uuid == default_cand.vg_uuid;

            let texture = icons
                .get(&cand.part_uuid)
                .and_then(|data| decode_icns_to_texture(data));

            let object = BootCandidateObject::new(cand);
            if let Some(texture) = texture {
                *object.imp().icon.borrow_mut() = Some(texture);
            }
            self.get_list_store().append(&object);

            if is_default {
                self.imp().changing_selection.set(true);
                self.imp()
                    .grid_view
                    .model()
                    .unwrap()
                    .select_item(idx as u32, true);
                self.imp().changing_selection.set(false);
                self.imp().confirmed_selection.set(idx as u32);
            }
        }
        Ok(())
    }
}

fn decode_icns_to_texture(data: &[u8]) -> Option<adw::gdk::Texture> {
    let icon_family = icns::IconFamily::read(Cursor::new(data)).ok()?;

    let image = icon_family
        .get_icon_with_type(icns::IconType::RGBA32_128x128)
        .or_else(|_| {
            // Fall back to the largest available icon
            let available = icon_family.available_icons();
            let largest = available
                .iter()
                .max_by_key(|t| t.pixel_width() * t.pixel_height())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no icons"))?;
            icon_family.get_icon_with_type(*largest)
        })
        .ok()?;

    let width = image.width();
    let height = image.height();
    let pixel_data = image.data();
    let stride = width * 4;

    let texture = adw::gdk::MemoryTexture::new(
        width as i32,
        height as i32,
        adw::gdk::MemoryFormat::R8g8b8a8,
        &adw::glib::Bytes::from(pixel_data),
        stride as usize,
    );

    Some(texture.into())
}
