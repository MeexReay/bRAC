use libadwaita::{glib, gtk};

use glib::object::ObjectExt;
use gtk::{subclass::prelude::*, prelude::LayoutManagerExt, BoxLayout};

#[derive(Debug)]
pub struct CustomLayout {
    box_layout: BoxLayout
}

impl Default for CustomLayout {
    fn default() -> Self {
        CustomLayout {
            box_layout: BoxLayout::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(5)
                .build()
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CustomLayout {
    const NAME: &'static str = "CustomLayout";
    type Type = super::CustomLayout;
    type ParentType = gtk::LayoutManager;
}

impl ObjectImpl for CustomLayout {
    fn signals() -> &'static [glib::subclass::Signal] {
        use std::sync::OnceLock;
        
        static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();

        SIGNALS.get_or_init(|| {
            vec![glib::subclass::Signal::builder("size-changed").build()]
        })
    }
}
impl LayoutManagerImpl for CustomLayout {
    fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
        self.obj().emit_by_name::<()>("size-changed", &[]);
        self.box_layout.allocate(widget, width, height, baseline)
    }
    fn measure(
            &self,
            widget: &gtk::Widget,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
        self.box_layout.measure(widget, orientation, for_size)
    }
}
