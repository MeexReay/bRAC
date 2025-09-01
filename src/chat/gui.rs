use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::hash::{DefaultHasher, Hasher};
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;
use std::sync::{atomic::Ordering, mpsc::channel, Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

use chrono::Local;
use clap::crate_version;

use libadwaita::gdk::Texture;
use libadwaita::gtk::gdk_pixbuf::InterpType;
use libadwaita::gtk::{Adjustment, MenuButton};
use libadwaita::{
    self as adw, ActionRow, Avatar, ButtonRow, EntryRow, HeaderBar, PreferencesDialog, PreferencesGroup, PreferencesPage, SpinRow, SwitchRow
};
use adw::gdk::{Cursor, Display};
use adw::gio::{ActionEntry, ApplicationFlags, MemoryInputStream, Menu};
use adw::glib::clone;
use adw::glib::{
    self, clone::Downgrade, source::timeout_add_local_once,
    timeout_add_local, timeout_add_once,
    ControlFlow,
};
use adw::prelude::*;
use adw::{Application, ApplicationWindow};

use adw::gtk;
use gtk::gdk_pixbuf::{Pixbuf, PixbufAnimation, PixbufLoader};
use gtk::pango::WrapMode;
use gtk::{
    Align, Box as GtkBox, Button, Calendar,
    CssProvider, Entry, Fixed, GestureClick, Justification, Label, ListBox,
    Orientation, Overlay, Picture, ScrolledWindow, Settings,
};

use crate::chat::grab_avatar;

use super::{
    config::{
        get_config_path, save_config, Config,
    },
    ctx::Context,
    on_send_message, parse_message, print_message, recv_tick, sanitize_message, SERVER_LIST,
};

pub fn try_save_config(path: PathBuf, config: &Config) {
    match save_config(path, config) {
        Ok(_) => {},
        Err(e) => {
            println!("save config error: {e}")
        }
    }
}

struct UiModel {
    is_dark_theme: bool,
    chat_box: GtkBox,
    chat_scrolled: ScrolledWindow,
    app: Application,
    window: ApplicationWindow,
    #[cfg(feature = "libnotify")]
    notifications: Arc<RwLock<Vec<libnotify::Notification>>>,
    #[cfg(all(not(feature = "libnotify"), not(feature = "notify-rust")))]
    notifications: Arc<RwLock<Vec<String>>>,
    avatars: Arc<Mutex<HashMap<u64, Vec<Avatar>>>>,
    latest_sign: Arc<AtomicU64>
}

thread_local!(
    static GLOBAL: RefCell<Option<UiModel>> = RefCell::new(None);
);

pub fn clear_chat_messages(ctx: Arc<Context>, messages: Vec<String>) {
    let _ = ctx
        .sender
        .read()
        .unwrap()
        .clone()
        .unwrap()
        .send((messages, true));
}

pub fn add_chat_messages(ctx: Arc<Context>, messages: Vec<String>) {
    println!("add chat messages: {}", messages.len());
    let _ = ctx
        .sender
        .read()
        .unwrap()
        .clone()
        .unwrap()
        .send((messages, false));
}

fn load_pixbuf(data: &[u8]) -> Result<Pixbuf, Box<dyn Error>> {
    let loader = PixbufLoader::new();
    loader.write(data)?;
    loader.close()?;
    Ok(loader.pixbuf().ok_or("laod pixbuf error")?)
}

fn update_window_title(ctx: Arc<Context>) {
    GLOBAL.with(|global| {
        if let Some(ui) = &*global.borrow() {
            ui.window.set_title(Some(&format!(
                "bRAC - Connected to {} as {}",
                ctx.config(|o| o.host.clone()),
                &ctx.name()
            )))
        }
    })
}

fn open_settings(ctx: Arc<Context>, app: &Application) {
    let dialog = PreferencesDialog::builder().build();


    let page = PreferencesPage::builder()
        .title("General")
        .icon_name("avatar-default-symbolic")
        .build();

    let group = PreferencesGroup::builder()
        .title("User Profile")
        .description("Profile preferences")
        .build();

    
    // Name preference

    let name = EntryRow::builder()
        .title("Name")
        .text(ctx.config(|o| o.name.clone()).unwrap_or_default())
        .build();

    group.add(&name);


    // Avatar preference
    
    let avatar = EntryRow::builder()
        .title("Avatar URL")
        .text(ctx.config(|o| o.avatar.clone()).unwrap_or_default())
        .build();

    group.add(&avatar);
    

    page.add(&group);



    let group = PreferencesGroup::builder()
        .title("Server")
        .description("Connection preferences")
        .build();

    
    // Host preference

    let host = EntryRow::builder()
        .title("Host")
        .text(ctx.config(|o| o.host.clone()))
        .build();

    group.add(&host);


    // Messages limit preference
    
    let messages_limit = SpinRow::builder()
        .title("Messages limit")
        .adjustment(&Adjustment::builder()
            .lower(1.0)
            .upper(1048576.0)
            .page_increment(10.0)
            .step_increment(10.0)
            .value(ctx.config(|o| o.max_messages) as f64)
            .build())
        .build();
    
    group.add(&messages_limit);

    
    // Update interval preference
    
    let update_interval = SpinRow::builder()
        .title("Update interval")
        .subtitle("In milliseconds")
        .adjustment(&Adjustment::builder()
            .lower(10.0)
            .upper(1048576.0)
            .page_increment(10.0)
            .step_increment(10.0)
            .value(ctx.config(|o| o.update_time) as f64)
            .build())
        .build();
    
    group.add(&update_interval);

    
    // Update interval OOF preference
    
    let update_interval_oof = SpinRow::builder()
        .title("Update interval when unfocused")
        .subtitle("In milliseconds")
        .adjustment(&Adjustment::builder()
            .lower(10.0)
            .upper(1048576.0)
            .page_increment(10.0)
            .step_increment(10.0)
            .value(ctx.config(|o| o.oof_update_time) as f64)
            .build())
        .build();

    group.add(&update_interval_oof);

    page.add(&group);


    
    let group = PreferencesGroup::builder()
        .title("Config")
        .description("Configuration tools")
        .build();

    let display = Display::default().unwrap();
    let clipboard = display.clipboard();

    let config_path = ActionRow::builder()
        .title("Config path")
        .subtitle(get_config_path().to_string_lossy())
        .css_classes(["property", "monospace"])
        .build();

    let config_path_copy = Button::from_icon_name("edit-copy-symbolic");

    config_path_copy.set_margin_top(10);
    config_path_copy.set_margin_bottom(10);
    config_path_copy.connect_clicked(clone!(
        #[weak] clipboard,
        move |_| {
            if let Some(text) = get_config_path().to_str() {
                clipboard.set_text(text);
            }
        }
    ));

    config_path.add_suffix(&config_path_copy);
    config_path.set_activatable(false);
    
    group.add(&config_path);

    // Reset button

    let reset_button = ButtonRow::builder()
        .title("Reset all")
        .build();

    reset_button.connect_activated(clone!(
        #[weak] ctx,
        #[weak] app,
        #[weak] dialog,
        move |_| {
            dialog.close();
            let config = Config::default();
            ctx.set_config(&config);
            try_save_config(get_config_path(), &config);
            open_settings(ctx, &app);
        }
    ));
    
    group.add(&reset_button);
    
    page.add(&group);

    dialog.add(&page);



    let page = PreferencesPage::builder()
        .title("Protocol")
        .icon_name("network-wired-symbolic")
        .build();

    let group = PreferencesGroup::builder()
        .title("Network")
        .description("Network preferences")
        .build();


    // Proxy preference

    let proxy = EntryRow::builder()
        .title("Socks proxy")
        .text(ctx.config(|o| o.proxy.clone()).unwrap_or_default())
        .build();

    group.add(&proxy);


    // Max avatar size preference
    
    let max_avatar_size = SpinRow::builder()
        .title("Max avatar size")
        .subtitle("Maximum avatar size in bytes")
        .adjustment(&Adjustment::builder()
            .lower(0.0)
            .upper(1074790400.0)
            .page_increment(1024.0)
            .step_increment(1024.0)
            .value(ctx.config(|o| o.max_avatar_size) as f64)
            .build())
        .build();

    group.add(&max_avatar_size);

    
    page.add(&group);


    let group = PreferencesGroup::builder()
        .title("Protocol")
        .description("Rac protocol preferences")
        .build();

    
    // Message format preference
    
    let message_format = EntryRow::builder()
        .title("Message format")
        .text(ctx.config(|o| o.message_format.clone()))
        .build();

    group.add(&message_format);

    page.add(&group);

    
    // Hide IP preference
    
    let hide_my_ip = SwitchRow::builder()
        .title("Hide IP")
        .subtitle("Hides only for clRAC and other dummy clients")
        .active(ctx.config(|o| o.hide_my_ip))
        .build();

    group.add(&hide_my_ip);


    // Chunked reading preference
    
    let chunked_reading = SwitchRow::builder()
        .title("Chunked reading")
        .subtitle("Read messages in chunks (less traffic usage, less compatibility)")
        .active(ctx.config(|o| o.chunked_enabled))
        .build();

    group.add(&chunked_reading);

    
    // Enable commands preference
    
    let enable_commands = SwitchRow::builder()
        .title("Enable commands")
        .subtitle("Enable slash commands (eg. /login) on client-side")
        .active(ctx.config(|o| o.commands_enabled))
        .build();

    group.add(&enable_commands);

    
    page.add(&group);
    
    dialog.add(&page);


    let page = PreferencesPage::builder()
        .title("Interface")
        .icon_name("applications-graphics-symbolic")
        .build();

    let group = PreferencesGroup::builder()
        .title("Messages")
        .description("Messages render preferences")
        .build();

    
    // Debug logs preference
    
    let debug_logs = SwitchRow::builder()
        .title("Debug logs")
        .subtitle("Print debug logs to the chat")
        .active(ctx.config(|o| o.debug_logs))
        .build();
    
    group.add(&debug_logs);

    
    // Show IPs preference
    
    let show_ips = SwitchRow::builder()
        .title("Show IPs")
        .subtitle("Show authors IP addresses if possible")
        .active(ctx.config(|o| o.show_other_ip))
        .build();
    
    group.add(&show_ips);

    
    // Format messages preference
    
    let format_messages = SwitchRow::builder()
        .title("Format messages")
        .subtitle("Disable to see raw messages")
        .active(ctx.config(|o| o.formatting_enabled))
        .build();
    
    group.add(&format_messages);

    
    // Show avatars preference
    
    let show_avatars = SwitchRow::builder()
        .title("Show avatars")
        .subtitle("Enables new messages UI")
        .active(ctx.config(|o| o.new_ui_enabled))
        .build();
    
    group.add(&show_avatars);
    page.add(&group);

    
    let group = PreferencesGroup::builder()
        .title("Interface")
        .description("General interface preferences (restart after changing)")
        .build();

    
    // Remove GUI shit preference
    
    let remove_gui_shit = SwitchRow::builder()
        .title("Remove GUI shit")
        .subtitle("Removes calendar, konata and clock")
        .active(ctx.config(|o| o.remove_gui_shit))
        .build();
    
    group.add(&remove_gui_shit);

    
    // Konata size preference
    
    let konata_size = SpinRow::builder()
        .title("Konata size")
        .subtitle("Set konata size percent")
        .adjustment(&Adjustment::builder()
            .lower(0.0)
            .upper(200.0)
            .page_increment(10.0)
            .step_increment(10.0)
            .value(ctx.config(|o| o.konata_size) as f64)
            .build())
        .build();

    group.add(&konata_size);
    
    
    // Enable notifications preference
    
    let enable_notifications = SwitchRow::builder()
        .title("Enable notifications")
        .subtitle("Send notifications on chat and system messages")
        .active(ctx.config(|o| o.notifications_enabled))
        .build();
    
    group.add(&enable_notifications);
    page.add(&group);
    
    
    dialog.add(&page);

    
    dialog.connect_closed(move |_| {
        let config = Config {
            host: host.text().to_string(),
            name: {
                let name = name.text().to_string();

                if name.is_empty() {
                    None
                } else {
                    Some(name)
                }
            },
            avatar: {
                let avatar = avatar.text().to_string();

                if avatar.is_empty() {
                    None
                } else {
                    Some(avatar)
                }
            },
            message_format: message_format.text().to_string(),
            update_time: update_interval.value() as usize,
            oof_update_time: update_interval_oof.value() as usize,
            konata_size: konata_size.value() as usize,
            max_messages: messages_limit.value() as usize,
            max_avatar_size: max_avatar_size.value() as u64,
            hide_my_ip: hide_my_ip.is_active(),
            remove_gui_shit: remove_gui_shit.is_active(),
            show_other_ip: show_ips.is_active(),
            chunked_enabled: chunked_reading.is_active(),
            formatting_enabled: format_messages.is_active(),
            commands_enabled: enable_commands.is_active(),
            notifications_enabled: enable_notifications.is_active(),
            new_ui_enabled: show_avatars.is_active(),
            debug_logs: debug_logs.is_active(),
            proxy: {
                let proxy = proxy.text().to_string();

                if proxy.is_empty() {
                    None
                } else {
                    Some(proxy)
                }
            },
        };
        ctx.set_config(&config);
        try_save_config(get_config_path(), &config);
        update_window_title(ctx.clone());
    });

    dialog.present(app.active_window().as_ref());
}

fn build_menu(ctx: Arc<Context>, app: &Application) -> Menu {
    let menu = Menu::new();

    menu.append(Some("Settings"), Some("app.settings"));
    menu.append(Some("About"), Some("app.about"));
    menu.append(Some("Close"), Some("app.close"));

    app.add_action_entries([
        ActionEntry::builder("settings")
            .activate(clone!(
                #[weak]
                ctx,
                move |a: &Application, _, _| {
                    open_settings(ctx, a);
                }
            ))
            .build(),
        ActionEntry::builder("close")
            .activate(move |a: &Application, _, _| {
                a.quit();
            })
            .build(),
        ActionEntry::builder("about")
            .activate(clone!(
                #[weak]
                app,
                move |_, _, _| {
                    let dialog = adw::AboutDialog::builder()
                        .developer_name("MeexReay")
                        .license(glib::markup_escape_text(include_str!("../../LICENSE")))
                        .comments("better RAC client")
                        .website("https://github.com/MeexReay/bRAC")
                        .application_name("bRAC")
                        .application_icon("ru.themixray.bRAC")
                        .version(crate_version!())
                        .build();
                    dialog.present(app.active_window().as_ref());
                }
            ))
            .build(),
    ]);

    menu
}

fn build_ui(ctx: Arc<Context>, app: &Application) -> UiModel {
    let is_dark_theme = if let Some(settings) = Settings::default() {
        settings.is_gtk_application_prefer_dark_theme()
            || settings
                .gtk_theme_name()
                .map(|o| o.to_lowercase().contains("dark"))
                .unwrap_or_default()
    } else {
        false
    };

    #[cfg(target_os = "windows")]
    let is_dark_theme = true;

    let main_box = GtkBox::new(Orientation::Vertical, 5);

    let header = HeaderBar::new();

    header.pack_end(&MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&build_menu(ctx.clone(), &app))
        .build());
    
    main_box.append(&header);

    main_box.set_css_classes(&["main-box"]);

    let widget_box_overlay = Overlay::new();

    let widget_box = GtkBox::new(Orientation::Horizontal, 5);

    widget_box.set_css_classes(&["widget_box"]);

    let remove_gui_shit = ctx.config(|c| c.remove_gui_shit);

    if !remove_gui_shit {
        widget_box.append(
            &Calendar::builder()
                .css_classes(["calendar"])
                .show_heading(false)
                .can_target(false)
                .build(),
        );
    }

    let server_list_vbox = GtkBox::new(Orientation::Vertical, 5);

    let server_list = ListBox::new();

    for url in SERVER_LIST.iter() {
        let url = url.to_string();

        let label = Label::builder().label(&url).halign(Align::Start).build();

        let click = GestureClick::new();

        click.connect_pressed(clone!(
            #[weak]
            ctx,
            move |_, _, _, _| {
                let mut config = ctx.config.read().unwrap().clone();
                config.host = url.clone();
                ctx.set_config(&config);
                try_save_config(get_config_path(), &config);
                update_window_title(ctx.clone());
            }
        ));

        label.add_controller(click);

        server_list.append(&label);
    }

    server_list_vbox.append(&Label::builder().label("Server List:").build());

    server_list_vbox.append(&server_list);

    widget_box.append(&server_list_vbox);

    if !remove_gui_shit {
        let fixed = Fixed::new();
        fixed.set_can_target(false);

        let konata_size = ctx.config(|c| c.konata_size) as i32;

        let konata =
            Picture::for_pixbuf(&load_pixbuf(include_bytes!("images/konata.png")).unwrap());
        konata.set_size_request(174 * konata_size / 100, 127 * konata_size / 100);

        fixed.put(
            &konata,
            (499 - 174 * konata_size / 100) as f64,
            (131 - 127 * konata_size / 100) as f64,
        );

        let logo_gif = include_bytes!("images/logo.gif");

        let logo = Picture::for_pixbuf(&load_pixbuf(logo_gif).unwrap());
        logo.set_size_request(152 * konata_size / 100, 64 * konata_size / 100);

        let logo_anim = PixbufAnimation::from_stream(
            &MemoryInputStream::from_bytes(&glib::Bytes::from(logo_gif)),
            None::<&adw::gtk::gio::Cancellable>,
        )
        .unwrap()
        .iter(Some(SystemTime::now()));

        timeout_add_local(Duration::from_millis(30), {
            let logo = logo.clone();
            let logo_anim = logo_anim.clone();
            let ctx = ctx.clone();

            move || {
                if ctx.is_focused.load(Ordering::SeqCst) {
                    logo.set_pixbuf(Some(&logo_anim.pixbuf()));
                    logo_anim.advance(SystemTime::now());
                }
                ControlFlow::Continue
            }
        });

        // 262, 4
        fixed.put(
            &logo,
            (436 - 174 * konata_size / 100) as f64,
            (131 - 127 * konata_size / 100) as f64,
        );

        let time = Label::builder()
            .label(&Local::now().format("%H:%M").to_string())
            .justify(Justification::Right)
            .css_classes(["time"])
            .build();

        timeout_add_local(Duration::from_secs(1), {
            let time = time.clone();

            move || {
                time.set_label(&Local::now().format("%H:%M").to_string());

                ControlFlow::Continue
            }
        });

        fixed.put(&time, 432.0, 4.0);
        fixed.set_halign(Align::End);

        widget_box_overlay.add_overlay(&fixed);
    }

    widget_box_overlay.set_child(Some(&widget_box));

    main_box.append(&widget_box_overlay);

    let chat_box = GtkBox::new(Orientation::Vertical, 2);

    chat_box.set_css_classes(&["chat-box"]);

    let chat_scrolled = ScrolledWindow::builder()
        .child(&chat_box)
        .vexpand(true)
        .hexpand(true)
        .margin_bottom(5)
        .margin_end(5)
        .margin_start(5)
        .propagate_natural_height(true)
        .build();

    main_box.append(&chat_scrolled);

    let send_box = GtkBox::new(Orientation::Horizontal, 5);

    send_box.set_margin_bottom(5);
    send_box.set_margin_end(5);
    send_box.set_margin_start(5);

    let text_entry = Entry::builder()
        .placeholder_text("Message")
        .css_classes(["send-button"])
        .hexpand(true)
        .build();

    send_box.append(&text_entry);

    let send_btn = Button::builder()
        .label("Send")
        .css_classes(["send-text"])
        .cursor(&Cursor::from_name("pointer", None).unwrap())
        .build();

    send_btn.connect_clicked(clone!(
        #[weak]
        text_entry,
        #[weak]
        ctx,
        move |_| {
            let text = text_entry.text().clone();

            if text.is_empty() {
                return;
            }

            text_entry.set_text("");

            thread::spawn({
                move || {
                    if let Err(e) = on_send_message(ctx.clone(), &text) {
                        if ctx.config(|o| o.debug_logs) {
                            let msg = format!("Send message error: {}", e.to_string()).to_string();
                            add_chat_messages(ctx.clone(), vec![msg]);
                        }
                    }
                }
            });
        }
    ));

    text_entry.connect_activate(clone!(
        #[weak]
        text_entry,
        #[weak]
        ctx,
        move |_| {
            let text = text_entry.text().clone();

            if text.is_empty() {
                return;
            }

            text_entry.set_text("");

            thread::spawn({
                move || {
                    if let Err(e) = on_send_message(ctx.clone(), &text) {
                        if ctx.config(|o| o.debug_logs) {
                            let msg = format!("Send message error: {}", e.to_string()).to_string();
                            add_chat_messages(ctx.clone(), vec![msg]);
                        }
                    }
                }
            });
        }
    ));

    send_box.append(&send_btn);

    main_box.append(&send_box);

    let scrolled_window_weak = Downgrade::downgrade(&chat_scrolled);

    timeout_add_local_once(Duration::ZERO, {
        let scrolled_window_weak = scrolled_window_weak.clone();

        move || {
            if let Some(o) = scrolled_window_weak.upgrade() {
                o.vadjustment()
                    .set_value(o.vadjustment().upper() - o.vadjustment().page_size());
            }
        }
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!(
            "bRAC - Connected to {} as {}",
            ctx.config(|o| o.host.clone()),
            &ctx.name()
        ))
        .default_width(500)
        .default_height(500)
        .resizable(true)
        .decorated(true)
        .show_menubar(true)
        .content(&main_box)
        .build();
    
    window.connect_default_width_notify({
        let scrolled_window_weak = scrolled_window_weak.clone();

        move |_| {
            let scrolled_window_weak = scrolled_window_weak.clone();
            timeout_add_local_once(Duration::ZERO, move || {
                if let Some(o) = scrolled_window_weak.upgrade() {
                    o.vadjustment()
                        .set_value(o.vadjustment().upper() - o.vadjustment().page_size());
                }
            });
        }
    });

    window.present();

    UiModel {
        is_dark_theme,
        chat_scrolled,
        chat_box,
        app: app.clone(),
        window: window.clone(),
        #[cfg(feature = "libnotify")]
        notifications: Arc::new(RwLock::new(Vec::<libnotify::Notification>::new())),
        #[cfg(all(not(feature = "libnotify"), not(feature = "notify-rust")))]
        notifications: Arc::new(RwLock::new(Vec::<String>::new())),
        avatars: Arc::new(Mutex::new(HashMap::new())),
        latest_sign: Arc::new(AtomicU64::new(0))
    }
}

fn setup(_: &Application, ctx: Arc<Context>, ui: UiModel) {
    let (sender, receiver) = channel();

    *ctx.sender.write().unwrap() = Some(Arc::new(sender));

    run_recv_loop(ctx.clone());

    ui.window.connect_notify(Some("is-active"), {
        let ctx = ctx.clone();

        move |a, _| {
            let is_focused = a.is_active();

            ctx.is_focused.store(is_focused, Ordering::SeqCst);

            if is_focused {
                thread::spawn({
                    let ctx = ctx.clone();
                    move || {
                        make_recv_tick(ctx.clone());
                    }
                });

                #[cfg(not(feature = "notify-rust"))]
                GLOBAL.with(|global| {
                    if let Some(ui) = &*global.borrow() {
                        #[cfg(feature = "libnotify")]
                        for i in ui.notifications.read().unwrap().clone() {
                            i.close().expect("libnotify close error");
                        }
                        #[cfg(not(feature = "libnotify"))]
                        for i in ui.notifications.read().unwrap().clone() {
                            ui.app.withdraw_notification(&i);
                        }
                    }
                });
            }
        }
    });

    GLOBAL.with(|global| {
        *global.borrow_mut() = Some(ui);
    });

    thread::spawn({
        let ctx = ctx.clone();
        move || {
            while let Ok((messages, clear)) = receiver.recv() {
                println!("got chat messages: {}", messages.len());
                let ctx = ctx.clone();
                let messages = Arc::new(messages);

                timeout_add_once(Duration::ZERO, {
                    let messages = messages.clone();

                    move || {
                        GLOBAL.with(|global| {
                            if let Some(ui) = &*global.borrow() {
                                if clear {
                                    while let Some(row) = ui.chat_box.last_child() {
                                        ui.chat_box.remove(&row);
                                    }
                                }

                                for message in messages.iter() {
                                    on_add_message(ctx.clone(), &ui, message.to_string(), !clear);
                                }
                            }
                        });

                        if ctx.config(|o| !o.new_ui_enabled) {
                            return;
                        }
                        
                        thread::spawn(move || {
                            for message in messages.iter() {
                                let Some(avatar_url) = grab_avatar(message) else { continue };
                                let avatar_id = get_avatar_id(&avatar_url);

                                let Some(avatar) = load_avatar(&avatar_url, ctx.config(|o| o.max_avatar_size as usize)) else { println!("cant load avatar: {avatar_url} request error"); continue };
                                let Ok(pixbuf) = load_pixbuf(&avatar) else { println!("cant load avatar: {avatar_url} pixbuf error"); continue; };
                                let Some(pixbuf) = pixbuf.scale_simple(32, 32, InterpType::Bilinear) else { println!("cant load avatar: {avatar_url} scale image error"); continue };
                                let texture = Texture::for_pixbuf(&pixbuf);

                                timeout_add_once(Duration::ZERO, {
                                    move || {
                                        GLOBAL.with(|global| {
                                            if let Some(ui) = &*global.borrow() {
                                                if let Some(pics) = ui.avatars.lock().unwrap().remove(&avatar_id) {
                                                    for pic in pics {
                                                        pic.set_custom_image(Some(&texture));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            }
                        });
                    }
                });
            }
        }
    });
}

fn load_css(is_dark_theme: bool) {
    let provider = CssProvider::new();
    provider.load_from_data(&format!(
        "{}\n{}",
        if is_dark_theme {
            include_str!("styles/dark.css")
        } else {
            include_str!("styles/light.css")
        },
        include_str!("styles/style.css")
    ));

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[cfg(feature = "notify-rust")]
fn send_notification(_: Arc<Context>, _: &UiModel, title: &str, message: &str) {
    use notify_rust::{Notification, Timeout};

    Notification::new()
        .summary(title)
        .body(message)
        .auto_icon()
        .appname("bRAC")
        .timeout(Timeout::Default) // this however is
        .show()
        .expect("notify-rust send error");
}

#[cfg(feature = "libnotify")]
fn send_notification(_: Arc<Context>, ui: &UiModel, title: &str, message: &str) {
    use libnotify::Notification;

    let notification = Notification::new(title, message, None);
    notification.set_app_name("bRAC");
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader
        .loader_write(include_bytes!("images/icon.png"))
        .unwrap();
    pixbuf_loader.close().unwrap();
    notification.set_image_from_pixbuf(&pixbuf_loader.get_pixbuf().unwrap());
    notification.show().expect("libnotify send error");

    ui.notifications.write().unwrap().push(notification);
}

#[cfg(all(not(feature = "libnotify"), not(feature = "notify-rust")))]
fn send_notification(_: Arc<Context>, ui: &UiModel, title: &str, message: &str) {
    use std::{
        hash::{DefaultHasher, Hasher},
        time::UNIX_EPOCH,
    };

    use gtk::gio::Notification;

    let mut hash = DefaultHasher::new();
    hash.write(title.as_bytes());
    hash.write(message.as_bytes());

    let id = format!(
        "bRAC-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        hash.finish()
    );

    let notif = Notification::new(title);
    notif.set_body(Some(&message));
    ui.app.send_notification(Some(&id), &notif);

    ui.notifications.write().unwrap().push(id);
}

fn get_message_box(
    ctx: Arc<Context>,
    ui: &UiModel,
    message: String,
    notify: bool,
    formatting_enabled: bool,
) -> GtkBox {
    // TODO: softcode these colors

    let (ip_color, date_color, text_color) = if ui.is_dark_theme {
        ("#494949", "#929292", "#FFFFFF")
    } else {
        ("#585858", "#292929", "#000000")
    };

    let mut label = String::new();

    if let (true, Some((date, ip, content, nick, _))) =
        (formatting_enabled, parse_message(message.clone()))
    {
        if let Some(ip) = ip {
            if ctx.config(|o| o.show_other_ip) {
                label.push_str(&format!(
                    "<span color=\"{ip_color}\">{}</span> ",
                    glib::markup_escape_text(&ip)
                ));
            }
        }

        label.push_str(&format!(
            "<span color=\"{date_color}\">[{}]</span> ",
            glib::markup_escape_text(&date)
        ));

        if let Some((name, color)) = nick {
            label.push_str(&format!(
                "<span font_weight=\"bold\" color=\"{}\">&lt;{}&gt;</span> ",
                color.to_uppercase(),
                glib::markup_escape_text(&name)
            ));

            if notify && !ui.window.is_active() {
                if ctx.config(|o| o.chunked_enabled) {
                    send_notification(
                        ctx.clone(),
                        ui,
                        &format!("{}'s Message", &name),
                        &glib::markup_escape_text(&content),
                    );
                }
            }
        } else {
            if notify && !ui.window.is_active() {
                if ctx.config(|o| o.chunked_enabled) {
                    send_notification(ctx.clone(), ui, "System Message", &content);
                }
            }
        }

        label.push_str(&format!(
            "<span color=\"{text_color}\">{}</span>",
            glib::markup_escape_text(&content)
        ));
    } else {
        label.push_str(&format!(
            "<span color=\"{text_color}\">{}</span>",
            glib::markup_escape_text(&message)
        ));

        if notify && !ui.window.is_active() {
            if ctx.config(|o| o.chunked_enabled) {
                send_notification(ctx.clone(), ui, "Chat Message", &message);
            }
        }
    }

    let hbox = GtkBox::new(Orientation::Horizontal, 2);

    hbox.append(
        &Label::builder()
            .label(&label)
            .halign(Align::Start)
            .valign(Align::Start)
            .selectable(true)
            .wrap(true)
            .wrap_mode(WrapMode::WordChar)
            .use_markup(true)
            .build(),
    );

    hbox.set_hexpand(true);

    hbox
}

fn get_avatar_id(url: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(url.as_bytes());
    hasher.finish()
}

fn load_avatar(url: &str, response_limit: usize) -> Option<Vec<u8>> {
    reqwest::blocking::get(url).ok()
        .and_then(|mut resp| {
            let mut data = Vec::new();
            let mut length = 0;
            
            loop {
                if length >= response_limit {
                    break;
                }
                let mut buf = vec![0; (response_limit - length).min(1024)];
                let now_len = resp.read(&mut buf).ok()?;
                if now_len == 0 {
                    break;
                }
                buf.truncate(now_len);
                length += now_len;
                data.append(&mut buf);
            }

            Some(data)
        })
}

fn get_new_message_box(
    ctx: Arc<Context>,
    ui: &UiModel,
    message: String,
    notify: bool,
    formatting_enabled: bool
) -> Overlay {
    // TODO: softcode these colors

    let (ip_color, date_color, text_color) = if ui.is_dark_theme {
        ("#494949", "#929292", "#FFFFFF")
    } else {
        ("#585858", "#292929", "#000000")
    };

    let latest_sign = ui.latest_sign.load(Ordering::SeqCst);

    let (date, ip, content, name, color, avatar) =
        if let (true, Some((date, ip, content, nick, avatar))) =
            (formatting_enabled, parse_message(message.clone()))
        {
            
            (
                date,
                ip,
                content,
                nick.as_ref()
                    .map(|o| o.0.to_string())
                    .unwrap_or("System".to_string()),
                nick.as_ref()
                    .map(|o| o.1.to_string())
                    .unwrap_or("#DDDDDD".to_string()),
                avatar.map(|o| get_avatar_id(&o)).unwrap_or_default()
            )
        } else {
            (
                Local::now().format("%d.%m.%Y %H:%M").to_string(),
                None,
                message,
                "System".to_string(),
                "#DDDDDD".to_string(),
                0
            )
        };
    
    if notify && !ui.window.is_active() {
        if ctx.config(|o| o.chunked_enabled) {
            send_notification(
                ctx.clone(),
                ui,
                &if name == "System" { 
                    "System Message".to_string()
                } else { 
                    format!("{}'s Message", name)
                },
                &glib::markup_escape_text(&content),
            );
        }
    }

    let sign = get_message_sign(&name, &date);

    let squashed = latest_sign == sign;

    ui.latest_sign.store(sign, Ordering::SeqCst);

    let overlay = Overlay::new();

    if !squashed {
        let fixed = Fixed::new();
        fixed.set_can_target(false);

        let avatar_picture = Avatar::builder()
            .text(&name)
            .show_initials(true)
            // .width_request(64)
            // .height_request(64)
            .size(32)
            .build();
        // avatar_picture.set_css_classes(&["message-avatar"]);
        avatar_picture.set_vexpand(false);
        avatar_picture.set_hexpand(false);
        avatar_picture.set_valign(Align::Start);
        avatar_picture.set_halign(Align::Start);
        // avatar_picture.set_size_request(64, 64);

        if avatar != 0 {
            let mut lock = ui.avatars.lock().unwrap();
            
            if let Some(pics) = lock.get_mut(&avatar) {
                pics.push(avatar_picture.clone());
            } else {
                lock.insert(avatar, vec![avatar_picture.clone()]);
            }
        }

        fixed.put(&avatar_picture, 0.0, 4.0);

        overlay.add_overlay(&fixed);
    }

    let vbox = GtkBox::new(Orientation::Vertical, 2);

    if !squashed {
        vbox.append(&Label::builder()
            .label(format!(
                "<span color=\"{color}\">{}</span> <span color=\"{date_color}\">{}</span> <span color=\"{ip_color}\">{}</span>", 
                glib::markup_escape_text(&name), 
                glib::markup_escape_text(&date),
                glib::markup_escape_text(&ip.unwrap_or_default()),
            ))
            .halign(Align::Start)
            .valign(Align::Start)
            .selectable(true)
            .wrap(true)
            .wrap_mode(WrapMode::WordChar)
            .use_markup(true)
            .build());
    }

    vbox.append(&Label::builder()
        .label(format!(
            "<span color=\"{text_color}\">{}</span>", 
            glib::markup_escape_text(&content)
        ))
        .halign(Align::Start)
        .hexpand(true)
        .selectable(true)
        .wrap(true)
        .wrap_mode(WrapMode::WordChar)
        .use_markup(true)
        .build());

    vbox.set_margin_start(37);
    vbox.set_hexpand(true);

    overlay.set_child(Some(&vbox));

    if !squashed {
        overlay.set_margin_top(7);
    } else {
        overlay.set_margin_top(2);
    }

    overlay
}

// creates sign that expires in 0-20 minutes
fn get_message_sign(name: &str, date: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(name.as_bytes());
    hasher.write(date[..date.len()-2].as_bytes());
    hasher.finish()
}

/// returns message sign
fn on_add_message(ctx: Arc<Context>, ui: &UiModel, message: String, notify: bool) {
    let notify = notify && ctx.config(|c| c.notifications_enabled);

    let formatting_enabled = ctx.config(|c| c.formatting_enabled);

    let Some(sanitized) = (if formatting_enabled {
        sanitize_message(message.clone())
    } else {
        Some(message.clone())
    }) else {
        return;
    };

    if sanitized.is_empty() {
        return;
    }

    if ctx.config(|o| o.new_ui_enabled) {
        ui.chat_box.append(&get_new_message_box(ctx.clone(), ui, message, notify, formatting_enabled));
    } else {
        ui.chat_box.append(&get_message_box(ctx.clone(), ui, message, notify, formatting_enabled));
    };

    timeout_add_local_once(Duration::from_millis(1000), move || {
        GLOBAL.with(|global| {
            if let Some(ui) = &*global.borrow() {
                let o = &ui.chat_scrolled;
                o.vadjustment()
                    .set_value(o.vadjustment().upper() - o.vadjustment().page_size());
            }
        });
    });
}

fn make_recv_tick(ctx: Arc<Context>) {
    if let Err(e) = recv_tick(ctx.clone()) {
        if ctx.config(|o| o.debug_logs) {
            let _ = print_message(
                ctx.clone(),
                format!("Print messages error: {}", e.to_string()).to_string(),
            );
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn run_recv_loop(ctx: Arc<Context>) {
    thread::spawn(move || loop {
        make_recv_tick(ctx.clone());

        thread::sleep(Duration::from_millis(
            if ctx.is_focused.load(Ordering::SeqCst) {
                ctx.config(|o| o.update_time) as u64
            } else {
                ctx.config(|o| o.oof_update_time) as u64
            },
        ));
    });
}

pub fn run_main_loop(ctx: Arc<Context>) {
    #[cfg(feature = "libnotify")]
    {
        libnotify::init("ru.themixray.bRAC").expect("libnotify init error");
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;
        env::set_var("GTK_THEME", "Adwaita:dark");
    }

    let application = Application::builder()
        .application_id("ru.themixray.bRAC")
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    application.connect_activate({
        let ctx = ctx.clone();

        move |app| {
            let ui = build_ui(ctx.clone(), app);
            load_css(ui.is_dark_theme);
            setup(app, ctx.clone(), ui);
        }
    });

    application.connect_startup({
        let ctx = ctx.clone();

        move |app| {
            build_menu(ctx.clone(), app);
        }
    });

    application.run_with_args::<&str>(&[]);

    #[cfg(feature = "libnotify")]
    {
        libnotify::uninit();
    }
}
