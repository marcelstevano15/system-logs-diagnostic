use adw::prelude::*;

use super::window;

pub fn build_ui(app: &adw::Application) {
    window::create_application_window(app);
}
