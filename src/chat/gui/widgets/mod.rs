mod imp;

use libadwaita::gtk::glib;
use libadwaita::gtk;

glib::wrapper! {
    pub struct CustomLayout(ObjectSubclass<imp::CustomLayout>)
        @extends gtk::LayoutManager;
}

impl Default for CustomLayout {
    fn default() -> Self {
        glib::Object::new()
    }
}
