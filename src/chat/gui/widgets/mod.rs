mod imp;

use libadwaita::gtk;
use libadwaita::gtk::glib;

glib::wrapper! {
    pub struct CustomLayout(ObjectSubclass<imp::CustomLayout>)
        @extends gtk::LayoutManager;
}

impl Default for CustomLayout {
    fn default() -> Self {
        glib::Object::new()
    }
}
