use crate::state::AppState;
use crate::ui::columns::create_text_column;
use crate::app::events;
use adw::prelude::*;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

pub fn create_application_window(app: &adw::Application) {
    let state = AppState::new();
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("System Logs Diagnostic")
        .default_width(1600)
        .default_height(1000)
        .build();

    let split = adw::NavigationSplitView::new();
    let sidebar = gtk::ListBox::new();
    sidebar.add_css_class("navigation-sidebar");

    for title in [
        "Live Logs",
        "Boot Logs",
        "Kernel",
        "Security",
        "Services",
        "Storage",
        "Networking",
    ] {
        sidebar.append(
            &adw::ActionRow::builder()
                .title(title)
                .activatable(true)
                .build(),
        );
    }

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search Logs")
        .hexpand(true)
        .build();
    header.set_title_widget(Some(&search));

    let refresh = gtk::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Refresh")
        .build();
    header.pack_start(&refresh);

    let menu = gio::Menu::new();
    menu.append(Some("About"), Some("win.about"));

    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .build();
    header.pack_end(&menu_button);

    toolbar.add_top_bar(&header);

    let status = adw::StatusPage::builder()
        .title("System Health")
        .description("Retrieve System Logs...")
        .icon_name("utilities-system-monitor-symbolic")
        .build();

    let store = gio::ListStore::new::<glib::BoxedAnyObject>();
    let selection = gtk::NoSelection::new(Some(store.clone()));
    let column_view = gtk::ColumnView::new(Some(selection));
    column_view.set_vexpand(true);
    column_view.set_hexpand(true);

    column_view.append_column(&create_text_column("Timestamp", "timestamp"));
    column_view.append_column(&create_text_column("Priority", "priority"));
    column_view.append_column(&create_text_column("Process", "process"));
    column_view.append_column(&create_text_column("Message", "message"));

    let scroll = gtk::ScrolledWindow::builder()
        .child(&column_view)
        .vexpand(true)
        .hexpand(true)
        .build();

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content_box.append(&status);
    content_box.append(&scroll);

    toolbar.set_content(Some(&content_box));
    split.set_sidebar(Some(&adw::NavigationPage::new(&sidebar, "Sections")));
    split.set_content(Some(&adw::NavigationPage::new(&toolbar, "Logs")));
    window.set_content(Some(&split));

    window.present();

    events::setup_lifecycle_events(
        &window,
        &sidebar,
        &search,
        &refresh,
        &status,
        &store,
        state,
    );
}

