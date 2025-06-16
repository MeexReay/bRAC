use std::sync::{atomic::Ordering, mpsc::channel, Arc, RwLock};
use std::cell::RefCell;
use std::time::{Duration, SystemTime};
use std::thread;
use std::error::Error;

use chrono::Local;

use gtk4::{self as gtk};

use gtk::gdk_pixbuf::{Pixbuf, PixbufAnimation, PixbufLoader};
use gtk::prelude::*;
use gtk::gdk::{Cursor, Display, Texture};
use gtk::gio::{self, ActionEntry, ApplicationFlags, MemoryInputStream, Menu};
use gtk::glib::clone;
use gtk::glib::{
    self, clone::Downgrade, 
    timeout_add_local, 
    source::timeout_add_local_once,
    ControlFlow,
    timeout_add_once
};
use gtk::pango::WrapMode;
use gtk::{
    AboutDialog, Align, Application, ApplicationWindow, Box as GtkBox, 
    Button, Calendar, CheckButton, CssProvider, Entry, Fixed, GestureClick, 
    Justification, Label, ListBox, Orientation, Overlay, Picture, ScrolledWindow, Settings, Window
};

use super::{config::{default_max_messages, default_update_time, default_konata_size, default_oof_update_time, get_config_path, save_config, Config}, 
ctx::Context, on_send_message, parse_message, print_message, recv_tick, sanitize_message, SERVER_LIST};

struct UiModel {
    is_dark_theme: bool,
    chat_box: GtkBox,
    chat_scrolled: ScrolledWindow,
    app: Application,
    window: ApplicationWindow,
    #[cfg(feature = "libnotify")]
    notifications: Arc<RwLock<Vec<libnotify::Notification>>>,
    #[cfg(not(feature = "libnotify"))]
    notifications: Arc<RwLock<Vec<String>>>
}

thread_local!(
    static GLOBAL: RefCell<Option<UiModel>> = RefCell::new(None);
);

pub fn clear_chat_messages(ctx: Arc<Context>, messages: Vec<String>) {
    let _ = ctx.sender.read().unwrap().clone().unwrap().send((messages, true));
}

pub fn add_chat_messages(ctx: Arc<Context>, messages: Vec<String>) {
    let _ = ctx.sender.read().unwrap().clone().unwrap().send((messages, false));
}

fn load_pixbuf(data: &[u8]) -> Result<Pixbuf, Box<dyn Error>> {
    let loader = PixbufLoader::new();
    loader.write(data)?;
    loader.close()?;
    Ok(loader.pixbuf().ok_or("laod pixbuf error")?)
}

macro_rules! gui_entry_setting {
    ($e:expr, $i:ident, $ctx:ident, $vbox:ident) => {
        {
            let hbox = GtkBox::new(Orientation::Horizontal, 5);

            hbox.append(&Label::builder()
                .label($e)
                .build());

            let entry = Entry::builder()
                .text(&$ctx.config(|o| o.$i.clone()))
                .build();

            hbox.append(&entry);

            $vbox.append(&hbox);

            entry
        }
    };
}

macro_rules! gui_usize_entry_setting {
    ($e:expr, $i:ident, $ctx:ident, $vbox:ident) => {
        {
            let hbox = GtkBox::new(Orientation::Horizontal, 5);

            hbox.append(&Label::builder()
                .label($e)
                .build());

            let entry = Entry::builder()
                .text(&$ctx.config(|o| o.$i.to_string()))
                .build();

            hbox.append(&entry);

            $vbox.append(&hbox);

            entry
        }
    };
}

macro_rules! gui_option_entry_setting {
    ($e:expr, $i:ident, $ctx:ident, $vbox:ident) => {
        {
            let hbox = GtkBox::new(Orientation::Horizontal, 5);

            hbox.append(&Label::builder()
                .label($e)
                .build());

            let entry = Entry::builder()
                .text(&$ctx.config(|o| o.$i.clone()).unwrap_or_default())
                .build();

            hbox.append(&entry);

            $vbox.append(&hbox);

            entry
        }
    };
}

macro_rules! gui_checkbox_setting {
    ($e:expr, $i:ident, $ctx:ident, $vbox:ident) => {
        {
            let hbox = GtkBox::new(Orientation::Horizontal, 5);

            hbox.append(&Label::builder()
                .label($e)
                .build());

            let entry = CheckButton::builder()
                .active($ctx.config(|o| o.$i))
                .build();

            hbox.append(&entry);

            $vbox.append(&hbox);

            entry
        }
    };
}

fn open_settings(ctx: Arc<Context>, app: &Application) {
    let vbox = GtkBox::new(Orientation::Vertical, 10);

    vbox.set_margin_bottom(15);
    vbox.set_margin_top(15);
    vbox.set_margin_start(15);
    vbox.set_margin_end(15);

    let settings_vbox = GtkBox::new(Orientation::Vertical, 10);

    let host_entry = gui_entry_setting!("Host", host, ctx, settings_vbox);
    let name_entry = gui_option_entry_setting!("Name", name, ctx, settings_vbox);
    let message_format_entry = gui_entry_setting!("Message Format", message_format, ctx, settings_vbox);
    let proxy_entry = gui_option_entry_setting!("Socks5 proxy", proxy, ctx, settings_vbox);
    let update_time_entry = gui_usize_entry_setting!("Update Time", update_time, ctx, settings_vbox);
    let oof_update_time_entry = gui_usize_entry_setting!("Out-of-focus Update Time", oof_update_time, ctx, settings_vbox);
    let max_messages_entry = gui_usize_entry_setting!("Max Messages", max_messages, ctx, settings_vbox);
    let hide_my_ip_entry = gui_checkbox_setting!("Hide My IP", hide_my_ip, ctx, settings_vbox);
    let show_other_ip_entry = gui_checkbox_setting!("Show Other IP", show_other_ip, ctx, settings_vbox);
    let auth_enabled_entry = gui_checkbox_setting!("Fake Auth Enabled", auth_enabled, ctx, settings_vbox);
    let chunked_enabled_entry = gui_checkbox_setting!("Chunked Enabled", chunked_enabled, ctx, settings_vbox);
    let formatting_enabled_entry = gui_checkbox_setting!("Formatting Enabled", formatting_enabled, ctx, settings_vbox);
    let commands_enabled_entry = gui_checkbox_setting!("Commands Enabled", commands_enabled, ctx, settings_vbox);
    let notifications_enabled_entry = gui_checkbox_setting!("Notifications Enabled", notifications_enabled, ctx, settings_vbox);
    let debug_logs_entry = gui_checkbox_setting!("Debug Logs", debug_logs, ctx, settings_vbox);
    let konata_size_entry = gui_usize_entry_setting!("Konata Size", konata_size, ctx, settings_vbox);
    let remove_gui_shit_entry = gui_checkbox_setting!("Remove Gui Shit", remove_gui_shit, ctx, settings_vbox);

    let scrollable = ScrolledWindow::builder()
        .child(&settings_vbox)
        .vexpand(true)
        .hexpand(true)
        .build();

    vbox.append(&scrollable);

    let save_button = Button::builder()
        .label("Save")
        .build();

    vbox.append(&save_button);

    save_button.connect_clicked(clone!(
        #[weak] ctx,
        #[weak] host_entry,
        #[weak] name_entry,
        #[weak] message_format_entry,
        #[weak] update_time_entry,
        #[weak] max_messages_entry,
        #[weak] hide_my_ip_entry,
        #[weak] show_other_ip_entry,
        #[weak] auth_enabled_entry,
        #[weak] chunked_enabled_entry,
        #[weak] formatting_enabled_entry,
        #[weak] commands_enabled_entry,
        #[weak] notifications_enabled_entry,
        #[weak] proxy_entry,
        #[weak] debug_logs_entry,
        #[weak] oof_update_time_entry,
        #[weak] konata_size_entry,
        #[weak] remove_gui_shit_entry,
        move |_| {
            let config = Config {
                host: host_entry.text().to_string(),
                name: {
                    let name = name_entry.text().to_string();
        
                    if name.is_empty() {
                        None
                    } else {
                        Some(name)
                    }
                },
                message_format: message_format_entry.text().to_string(),
                update_time: {
                    let update_time = update_time_entry.text();
        
                    if let Ok(update_time) = update_time.parse::<usize>() {
                        update_time
                    } else {
                        let update_time = default_update_time();
                        update_time_entry.set_text(&update_time.to_string());
                        update_time
                    }
                },
                oof_update_time: {
                    let oof_update_time = oof_update_time_entry.text();
        
                    if let Ok(oof_update_time) = oof_update_time.parse::<usize>() {
                        oof_update_time
                    } else {
                        let oof_update_time = default_oof_update_time();
                        oof_update_time_entry.set_text(&oof_update_time.to_string());
                        oof_update_time
                    }
                },
                konata_size: {
                    let konata_size = konata_size_entry.text();
        
                    if let Ok(konata_size) = konata_size.parse::<usize>() {
                        konata_size.max(0).min(200)
                    } else {
                        let konata_size = default_konata_size();
                        konata_size_entry.set_text(&konata_size.to_string());
                        konata_size
                    }
                },
                max_messages: {
                    let max_messages = max_messages_entry.text();
        
                    if let Ok(max_messages) = max_messages.parse::<usize>() {
                        max_messages
                    } else {
                        let max_messages = default_max_messages();
                        max_messages_entry.set_text(&max_messages.to_string());
                        max_messages
                    }
                },
                hide_my_ip: hide_my_ip_entry.is_active(),
                remove_gui_shit: remove_gui_shit_entry.is_active(),
                show_other_ip: show_other_ip_entry.is_active(),
                auth_enabled: auth_enabled_entry.is_active(),
                chunked_enabled: chunked_enabled_entry.is_active(),
                formatting_enabled: formatting_enabled_entry.is_active(),
                commands_enabled: commands_enabled_entry.is_active(),
                notifications_enabled: notifications_enabled_entry.is_active(),
                debug_logs: debug_logs_entry.is_active(),
                proxy: {
                    let proxy = proxy_entry.text().to_string();
        
                    if proxy.is_empty() {
                        None
                    } else {
                        Some(proxy)
                    }
                }
            };
            ctx.set_config(&config);
            save_config(get_config_path(), &config);
        }
    ));

    let reset_button = Button::builder()
        .label("Reset all")
        .build();

    vbox.append(&reset_button);

    reset_button.connect_clicked(clone!(
        #[weak] ctx,
        #[weak] host_entry,
        #[weak] name_entry,
        #[weak] message_format_entry,
        #[weak] update_time_entry,
        #[weak] max_messages_entry,
        #[weak] hide_my_ip_entry,
        #[weak] show_other_ip_entry,
        #[weak] auth_enabled_entry,
        #[weak] chunked_enabled_entry,
        #[weak] formatting_enabled_entry,
        #[weak] commands_enabled_entry,
        #[weak] notifications_enabled_entry,
        #[weak] proxy_entry,
        move |_| {
            let config = Config::default();
            ctx.set_config(&config);
            save_config(get_config_path(), &config);
            host_entry.set_text(&config.host);
            name_entry.set_text(&config.name.unwrap_or_default());
            proxy_entry.set_text(&config.proxy.unwrap_or_default());
            message_format_entry.set_text(&config.message_format);
            update_time_entry.set_text(&config.update_time.to_string());
            max_messages_entry.set_text(&config.max_messages.to_string());
            hide_my_ip_entry.set_active(config.hide_my_ip);
            show_other_ip_entry.set_active(config.show_other_ip);
            auth_enabled_entry.set_active(config.auth_enabled);
            chunked_enabled_entry.set_active(config.chunked_enabled);
            formatting_enabled_entry.set_active(config.formatting_enabled);
            commands_enabled_entry.set_active(config.commands_enabled);
            notifications_enabled_entry.set_active(config.notifications_enabled);
        }
    ));
    let window = Window::builder()
        .application(app)
        .title("Settings")
        .default_width(400)
        .default_height(500)
        .resizable(true)
        .decorated(true)
        .child(&vbox)
        .build();

    let controller = gtk::EventControllerKey::new();
    controller.connect_key_pressed({
        let window = window.clone();

        move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                window.close();
                gtk::glib::Propagation::Proceed
            } else {
                gtk::glib::Propagation::Stop
            }
        }
    });

    window.add_controller(controller);
    
    window.present();
}

fn build_menu(ctx: Arc<Context>, app: &Application) {
    let menu = Menu::new();

    let file_menu = Menu::new();
    file_menu.append(Some("About"), Some("app.about"));
    file_menu.append(Some("Close"), Some("app.close"));

    let edit_menu = Menu::new();
    edit_menu.append(Some("Settings"), Some("app.settings"));

    menu.append_submenu(Some("File"), &file_menu);
    menu.append_submenu(Some("Edit"), &edit_menu);

    app.set_menubar(Some((&menu).into()));

    app.add_action_entries([
        ActionEntry::builder("settings")
            .activate(clone!(
                #[weak] ctx,
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
                #[weak] app,
                move |_, _, _| {
                     AboutDialog::builder()
                        .application(&app)
                        .authors(["MeexReay"])
                        .license("        DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE 
                    Version 2, December 2004 

 Copyright (C) 2004 Sam Hocevar <sam@hocevar.net> 

 Everyone is permitted to copy and distribute verbatim or modified 
 copies of this license document, and changing it is allowed as long 
 as the name is changed. 

            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE 
   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION 

  0. You just DO WHAT THE FUCK YOU WANT TO.")
                        .comments("better RAC client")
                        .website("https://github.com/MeexReay/bRAC")
                        .website_label("source code")
                        .logo(&Texture::for_pixbuf(&load_pixbuf(include_bytes!("images/icon.png")).unwrap()))
                        .build()
                        .present();
                }
            ))
            .build()
    ]);
}

fn build_ui(ctx: Arc<Context>, app: &Application) -> UiModel {
    let is_dark_theme = if let Some(settings) = Settings::default() {
        settings.is_gtk_application_prefer_dark_theme() || settings.gtk_theme_name()
            .map(|o| o.to_lowercase().contains("dark"))
            .unwrap_or_default()
    } else {
        false
    };

    let main_box = GtkBox::new(Orientation::Vertical, 5);

    main_box.set_css_classes(&["main-box"]);

    let widget_box_overlay = Overlay::new();

    let widget_box = GtkBox::new(Orientation::Horizontal, 5);

    widget_box.set_css_classes(&["widget_box"]);

    let remove_gui_shit = ctx.config(|c| c.remove_gui_shit);

    if !remove_gui_shit {
        widget_box.append(&Calendar::builder()
            .css_classes(["calendar"])
            .show_heading(false)
            .can_target(false)
            .build());
    }

    let server_list_vbox = GtkBox::new(Orientation::Vertical, 5);

    let server_list = ListBox::new();

    for url in SERVER_LIST.iter() {
        let url = url.to_string();

        let label = Label::builder()
            .label(&url)
            .halign(Align::Start)
            .build();

        let click = GestureClick::new();

        click.connect_pressed(clone!(
            #[weak] ctx,
            move |_, _, _, _| {
                let mut config = ctx.config.read().unwrap().clone();
                config.host = url.clone();
                ctx.set_config(&config);
                save_config(get_config_path(), &config);
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

        let konata = Picture::for_pixbuf(&load_pixbuf(include_bytes!("images/konata.png")).unwrap());
        konata.set_size_request(174 * konata_size / 100, 127 * konata_size / 100);
        
        fixed.put(&konata, (499 - 174 * konata_size / 100) as f64, (131 - 127 * konata_size / 100) as f64);

        let logo_gif = include_bytes!("images/logo.gif");

        let logo = Picture::for_pixbuf(&load_pixbuf(logo_gif).unwrap());
        logo.set_size_request(152 * konata_size / 100, 64 * konata_size / 100);

        let logo_anim = PixbufAnimation::from_stream(
            &MemoryInputStream::from_bytes(
                &glib::Bytes::from(logo_gif)
            ),
            None::<&gio::Cancellable>
        ).unwrap().iter(Some(SystemTime::now()));

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
        fixed.put(&logo, (436 - 174 * konata_size / 100) as f64, (131 - 127 * konata_size / 100) as f64);

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
        #[weak] text_entry,
        #[weak] ctx,
        move |_| {
            if text_entry.text().is_empty() { return; }
            timeout_add_local_once(Duration::ZERO, clone!(
                #[weak] text_entry,
                move || {
                    text_entry.set_text("");
                }
            ));

            if let Err(e) = on_send_message(ctx.clone(), &text_entry.text()) {
                if ctx.config(|o| o.debug_logs) {
                    let msg = format!("Send message error: {}", e.to_string()).to_string();
                    add_chat_messages(ctx.clone(), vec![msg]);
                }
            }
        }
    ));

    text_entry.connect_activate(clone!(
        #[weak] text_entry,
        #[weak] ctx,
        move |_| {
            if text_entry.text().is_empty() { return; }
            timeout_add_local_once(Duration::ZERO, clone!(
                #[weak] text_entry,
                move || {
                    text_entry.set_text("");
                }
            ));

            if let Err(e) = on_send_message(ctx.clone(), &text_entry.text()) {
                if ctx.config(|o| o.debug_logs) {
                    let msg = format!("Send message error: {}", e.to_string()).to_string();
                    add_chat_messages(ctx.clone(), vec![msg]);
                }
            }
        }
    ));

    send_box.append(&send_btn);

    main_box.append(&send_box);

    let scrolled_window_weak = Downgrade::downgrade(&chat_scrolled);

    timeout_add_local_once(Duration::ZERO, {
        let scrolled_window_weak = scrolled_window_weak.clone();
        
        move || {
            if let Some(o) = scrolled_window_weak.upgrade() {
                o.vadjustment().set_value(o.vadjustment().upper() - o.vadjustment().page_size());
            }
        }
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("bRAC - Connected to {} as {}", ctx.config(|o| o.host.clone()), &ctx.name()))
        .default_width(500)
        .default_height(500)
        .resizable(true)
        .decorated(true)
        .show_menubar(true)
        .child(&main_box)
        .build();

    window.connect_default_width_notify({
        let scrolled_window_weak = scrolled_window_weak.clone();

        move |_| {
            let scrolled_window_weak = scrolled_window_weak.clone();
            timeout_add_local_once(Duration::ZERO, move || {
                if let Some(o) = scrolled_window_weak.upgrade() {
                    o.vadjustment().set_value(o.vadjustment().upper() - o.vadjustment().page_size());
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
        #[cfg(not(feature = "libnotify"))]
        notifications: Arc::new(RwLock::new(Vec::<String>::new())),
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
                let ctx = ctx.clone();
                timeout_add_once(Duration::ZERO, move || {
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

#[cfg(feature = "libnotify")]
fn send_notification(_: Arc<Context>, ui: &UiModel, title: &str, message: &str) {
    use libnotify::Notification;

    let notification = Notification::new(title, message, None);
    notification.set_app_name("bRAC");
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    pixbuf_loader.loader_write(include_bytes!("images/icon.png")).unwrap();
    pixbuf_loader.close().unwrap();
    notification.set_image_from_pixbuf(&pixbuf_loader.get_pixbuf().unwrap());
    notification.show().expect("libnotify send error");

    ui.notifications.write().unwrap().push(notification);
}

#[cfg(not(feature = "libnotify"))]
fn send_notification(_: Arc<Context>, ui: &UiModel, title: &str, message: &str) {
    use std::{hash::{DefaultHasher, Hasher}, time::UNIX_EPOCH};

    use gtk4::gio::Notification;

    let mut hash = DefaultHasher::new();
    hash.write(title.as_bytes());
    hash.write(message.as_bytes());

    let id = format!("bRAC-{}-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(), hash.finish());

    let notif = Notification::new(title);
    notif.set_body(Some(&message));
    ui.app.send_notification(Some(&id), &notif);

    ui.notifications.write().unwrap().push(id);
}

fn on_add_message(ctx: Arc<Context>, ui: &UiModel, message: String, notify: bool) {
    let Some(message) = sanitize_message(message) else { return; };

    if message.is_empty() {
        return;
    }

    // TODO: softcode these colors

    let (ip_color, date_color, text_color) = if ui.is_dark_theme {
        (
            "#494949",
            "#929292",
            "#FFFFFF"
        )
    } else {
        (
            "#585858",
            "#292929",
            "#000000"
        )
    };

    let mut label = String::new();

    if let Some((date, ip, content, nick)) = parse_message(message.clone()) {
        if let Some(ip) = ip {
            if ctx.config(|o| o.show_other_ip) {
                label.push_str(&format!("<span color=\"{ip_color}\">{}</span> ", glib::markup_escape_text(&ip)));
            }
        }

        label.push_str(&format!("<span color=\"{date_color}\">[{}]</span> ", glib::markup_escape_text(&date)));

        if let Some((name, color)) = nick {
            label.push_str(&format!("<span font_weight=\"bold\" color=\"{}\">&lt;{}&gt;</span> ", color.to_uppercase(), glib::markup_escape_text(&name)));

            if notify && !ui.window.is_active() {
                if ctx.config(|o| o.chunked_enabled) {
                    send_notification(ctx.clone(), ui, &format!("{}'s Message", &name), &glib::markup_escape_text(&content));
                }
            }
        } else {
            if notify && !ui.window.is_active() {
                if ctx.config(|o| o.chunked_enabled) {
                    send_notification(ctx.clone(), ui, "System Message", &content);
                }
            }
        }

        label.push_str(&format!("<span color=\"{text_color}\">{}</span>", glib::markup_escape_text(&content)));
    } else {
        label.push_str(&format!("<span color=\"{text_color}\">{}</span>", glib::markup_escape_text(&message)));

        if notify && !ui.window.is_active() {
            if ctx.config(|o| o.chunked_enabled) {
                send_notification(ctx.clone(), ui, "Chat Message", &message);
            }
        }
    }
    
    let hbox = GtkBox::new(Orientation::Horizontal, 2);

    hbox.append(&Label::builder()
        .label(&label)
        .halign(Align::Start)
        .valign(Align::Start)
        .selectable(true)
        .wrap(true)
        .wrap_mode(WrapMode::WordChar)
        .use_markup(true)
        .build());

    hbox.set_hexpand(true);

    ui.chat_box.append(&hbox);

    timeout_add_local_once(Duration::from_millis(1000), move || {
        GLOBAL.with(|global| {
            if let Some(ui) = &*global.borrow() {
                let o = &ui.chat_scrolled;
                o.vadjustment().set_value(o.vadjustment().upper() - o.vadjustment().page_size());
            }
        });
    });
}

fn make_recv_tick(ctx: Arc<Context>) {
    if let Err(e) = recv_tick(ctx.clone()) {
        if ctx.config(|o| o.debug_logs) {
            let _ = print_message(ctx.clone(), format!("Print messages error: {}", e.to_string()).to_string());
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn run_recv_loop(ctx: Arc<Context>) {
    thread::spawn(move || {
        loop { 
            make_recv_tick(ctx.clone());

            thread::sleep(Duration::from_millis(
                if ctx.is_focused.load(Ordering::SeqCst) { 
                    ctx.config(|o| o.update_time) as u64 
                } else {
                    ctx.config(|o| o.oof_update_time) as u64 
                }
            ));
        }
    });
}

pub fn run_main_loop(ctx: Arc<Context>) {
    #[cfg(feature = "libnotify")]
    {
        libnotify::init("ru.themixray.bRAC").expect("libnotify init error");
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
