use std::sync::{Arc, mpsc::{channel, Receiver}};
use std::cell::RefCell;
use std::time::{Duration, SystemTime};
use std::thread;

use chrono::Local;

use gtk4::{
    self as gtk, gdk::{Cursor, Display, Texture}, gdk_pixbuf::{Pixbuf, PixbufAnimation, PixbufLoader}, gio::{
        self, ActionEntry, ApplicationFlags, 
        MemoryInputStream, Menu
    }, glib::{
        self, clone, clone::Downgrade, idle_add_local, idle_add_local_once, source::timeout_add_local_once, timeout_add_local, ControlFlow
    }, pango::WrapMode, prelude::*, AboutDialog, Align, Application, ApplicationWindow, Box as GtkBox, 
    Button, Calendar, CheckButton, CssProvider, Entry, Fixed, Justification, Label, ListBox, Orientation, 
    Overlay, Picture, ScrolledWindow, Settings, Window
};

use super::{config::{default_max_messages, default_update_time, get_config_path, save_config, Config}, 
ctx::Context, on_send_message, parse_message, print_message, recv_tick};

struct UiModel {
    chat_box: GtkBox,
    chat_scrolled: ScrolledWindow
}

thread_local!(
    static GLOBAL: RefCell<Option<(UiModel, Receiver<String>)>> = RefCell::new(None);
);

pub fn add_chat_message(ctx: Arc<Context>, message: String) {
    let _ = ctx.sender.read().unwrap().clone().unwrap().send(message);
}

fn load_pixbuf(data: &[u8]) -> Pixbuf {
    let loader = PixbufLoader::new();
    loader.write(data).unwrap();
    loader.close().unwrap();
    loader.pixbuf().unwrap()
}

// chunked_enabled: bool
// formatting_enabled: bool
// commands_enabled: bool

fn open_settings(ctx: Arc<Context>, app: &Application) {
    let vbox = GtkBox::new(Orientation::Vertical, 10);

    vbox.set_margin_bottom(15);
    vbox.set_margin_top(15);
    vbox.set_margin_start(15);
    vbox.set_margin_end(15);

    let host_hbox = GtkBox::new(Orientation::Horizontal, 5);

    host_hbox.append(&Label::builder()
        .label("Host")
        .build());

    let host_entry = Entry::builder()
        .text(&ctx.config(|o| o.host.clone()))
        .build();

    host_hbox.append(&host_entry);

    vbox.append(&host_hbox);

    let name_hbox = GtkBox::new(Orientation::Horizontal, 5);

    name_hbox.append(&Label::builder()
        .label("Name")
        .build());

    let name_entry = Entry::builder()
        .text(&ctx.config(|o| o.name.clone()).unwrap_or_default())
        .build();

    name_hbox.append(&name_entry);

    vbox.append(&name_hbox);

    let message_format_hbox = GtkBox::new(Orientation::Horizontal, 5);

    message_format_hbox.append(&Label::builder()
        .label("Message Format")
        .build());

    let message_format_entry = Entry::builder()
        .text(&ctx.config(|o| o.message_format.clone()))
        .build();

    message_format_hbox.append(&message_format_entry);

    vbox.append(&message_format_hbox);

    let update_time_hbox = GtkBox::new(Orientation::Horizontal, 5);

    update_time_hbox.append(&Label::builder()
        .label("Update Time")
        .build());

    let update_time_entry = Entry::builder()
        .text(&ctx.config(|o| o.update_time.to_string()))
        .build();

    update_time_hbox.append(&update_time_entry);

    vbox.append(&update_time_hbox);

    let max_messages_hbox = GtkBox::new(Orientation::Horizontal, 5);

    max_messages_hbox.append(&Label::builder()
        .label("Max Messages")
        .build());

    let max_messages_entry = Entry::builder()
        .text(&ctx.config(|o| o.max_messages.to_string()))
        .build();

    max_messages_hbox.append(&max_messages_entry);

    vbox.append(&max_messages_hbox);

    let max_messages_hbox = GtkBox::new(Orientation::Horizontal, 5);

    max_messages_hbox.append(&Label::builder()
        .label("Max Messages")
        .build());

    let max_messages_entry = Entry::builder()
        .text(&ctx.config(|o| o.max_messages.to_string()))
        .build();

    max_messages_hbox.append(&max_messages_entry);

    vbox.append(&max_messages_hbox);

    let hide_my_ip_hbox = GtkBox::new(Orientation::Horizontal, 5);

    hide_my_ip_hbox.append(&Label::builder()
        .label("Hide My IP")
        .build());

    let hide_my_ip_entry = CheckButton::builder()
        .active(ctx.config(|o| o.hide_my_ip))
        .build();

    hide_my_ip_hbox.append(&hide_my_ip_entry);

    vbox.append(&hide_my_ip_hbox);

    let show_other_ip_hbox = GtkBox::new(Orientation::Horizontal, 5);

    show_other_ip_hbox.append(&Label::builder()
        .label("Show Other IP")
        .build());

    let show_other_ip_entry = CheckButton::builder()
        .active(ctx.config(|o| o.show_other_ip))
        .build();

    show_other_ip_hbox.append(&show_other_ip_entry);

    vbox.append(&show_other_ip_hbox);

    let auth_enabled_hbox = GtkBox::new(Orientation::Horizontal, 5);

    auth_enabled_hbox.append(&Label::builder()
        .label("Auth Enabled")
        .build());

    let auth_enabled_entry = CheckButton::builder()
        .active(ctx.config(|o| o.auth_enabled))
        .build();

    auth_enabled_hbox.append(&auth_enabled_entry);

    vbox.append(&auth_enabled_hbox);

    let ssl_enabled_hbox = GtkBox::new(Orientation::Horizontal, 5);

    ssl_enabled_hbox.append(&Label::builder()
        .label("SSL Enabled")
        .build());

    let ssl_enabled_entry = CheckButton::builder()
        .active(ctx.config(|o| o.ssl_enabled))
        .build();

    ssl_enabled_hbox.append(&ssl_enabled_entry);

    vbox.append(&ssl_enabled_hbox);

    let chunked_enabled_hbox = GtkBox::new(Orientation::Horizontal, 5);

    chunked_enabled_hbox.append(&Label::builder()
        .label("Chunked Enabled")
        .build());

    let chunked_enabled_entry = CheckButton::builder()
        .active(ctx.config(|o| o.chunked_enabled))
        .build();

    chunked_enabled_hbox.append(&chunked_enabled_entry);

    vbox.append(&chunked_enabled_hbox);

    let formatting_enabled_hbox = GtkBox::new(Orientation::Horizontal, 5);

    formatting_enabled_hbox.append(&Label::builder()
        .label("Formatting Enabled")
        .build());

    let formatting_enabled_entry = CheckButton::builder()
        .active(ctx.config(|o| o.formatting_enabled))
        .build();

    formatting_enabled_hbox.append(&formatting_enabled_entry);

    vbox.append(&formatting_enabled_hbox);

    let commands_enabled_hbox = GtkBox::new(Orientation::Horizontal, 5);

    commands_enabled_hbox.append(&Label::builder()
        .label("Commands Enabled")
        .build());

    let commands_enabled_entry = CheckButton::builder()
        .active(ctx.config(|o| o.commands_enabled))
        .build();

    commands_enabled_hbox.append(&commands_enabled_entry);

    vbox.append(&commands_enabled_hbox);

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
        #[weak] ssl_enabled_entry,
        #[weak] chunked_enabled_entry,
        #[weak] formatting_enabled_entry,
        #[weak] commands_enabled_entry,
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
                show_other_ip: show_other_ip_entry.is_active(),
                auth_enabled: auth_enabled_entry.is_active(),
                ssl_enabled: ssl_enabled_entry.is_active(),
                chunked_enabled: chunked_enabled_entry.is_active(),
                formatting_enabled: formatting_enabled_entry.is_active(),
                commands_enabled: commands_enabled_entry.is_active()
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
        #[weak] ssl_enabled_entry,
        #[weak] chunked_enabled_entry,
        #[weak] formatting_enabled_entry,
        #[weak] commands_enabled_entry,
        move |_| {
            let config = Config::default();
            ctx.set_config(&config);
            save_config(get_config_path(), &config);
            host_entry.set_text(&config.host);
            name_entry.set_text(&config.name.unwrap_or_default());
            message_format_entry.set_text(&config.message_format);
            update_time_entry.set_text(&config.update_time.to_string());
            max_messages_entry.set_text(&config.max_messages.to_string());
            hide_my_ip_entry.set_active(config.hide_my_ip);
            show_other_ip_entry.set_active(config.show_other_ip);
            auth_enabled_entry.set_active(config.auth_enabled);
            ssl_enabled_entry.set_active(config.ssl_enabled);
            chunked_enabled_entry.set_active(config.chunked_enabled);
            formatting_enabled_entry.set_active(config.formatting_enabled);
            commands_enabled_entry.set_active(config.commands_enabled);
        }
    ));

    let window = Window::builder()
        .application(app)
        .title("Settings")
        .default_width(400)
        .default_height(500)
        .decorated(true)
        .child(&vbox)
        .build();

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
                        .authors(["TheMixRay", "MeexReay"])
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
                        .logo(&Texture::for_pixbuf(&load_pixbuf(include_bytes!("images/icon.png"))))
                        .build()
                        .present();
                }
            ))
            .build()
    ]);
}

fn build_ui(ctx: Arc<Context>, app: &Application) -> UiModel {
    let main_box = GtkBox::new(Orientation::Vertical, 5);

    main_box.set_css_classes(&["main-box"]);

    let widget_box_overlay = Overlay::new();

    let widget_box = GtkBox::new(Orientation::Horizontal, 5);

    widget_box.set_css_classes(&["widget_box"]);

    widget_box.append(&Calendar::builder()
        .css_classes(["calendar"])
        .show_heading(false)
        .can_target(false)
        .build());

    let server_list_vbox = GtkBox::new(Orientation::Vertical, 5);

    let server_list = ListBox::new();

    server_list.append(&Label::builder().label("meex.lol:42666").halign(Align::Start).selectable(true).build());
    server_list.append(&Label::builder().label("meex.lol:11234").halign(Align::Start).selectable(true).build());
    server_list.append(&Label::builder().label("91.192.22.20:42666").halign(Align::Start).selectable(true).build());

    server_list_vbox.append(&Label::builder().label("Server List:").build());

    server_list_vbox.append(&server_list);

    widget_box.append(&server_list_vbox);

    let fixed = Fixed::new();
    fixed.set_can_target(false);

    let konata = Picture::for_pixbuf(&load_pixbuf(include_bytes!("images/konata.png")));
    konata.set_size_request(174, 127);
    
    fixed.put(&konata, 325.0, 4.0);

    let logo_gif = include_bytes!("images/logo.gif");

    let logo = Picture::for_pixbuf(&load_pixbuf(logo_gif));
    logo.set_size_request(152, 64);

    let logo_anim = PixbufAnimation::from_stream(
        &MemoryInputStream::from_bytes(
            &glib::Bytes::from(logo_gif)
        ),
        None::<&gio::Cancellable>
    ).unwrap().iter(Some(SystemTime::now()));

    timeout_add_local(Duration::from_millis(30), {
        let logo = logo.clone();
        let logo_anim = logo_anim.clone();

        move || {
            logo.set_pixbuf(Some(&logo_anim.pixbuf()));
            logo_anim.advance(SystemTime::now());

            ControlFlow::Continue
        }
    });
    
    fixed.put(&logo, 262.0, 4.0);

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

    widget_box_overlay.add_overlay(&fixed);

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
            idle_add_local_once(clone!(
                #[weak] text_entry,
                move || {
                    text_entry.set_text("");
                }
            ));

            if let Err(e) = on_send_message(ctx.clone(), &text_entry.text()) {
                let msg = format!("Send message error: {}", e.to_string()).to_string();
                add_chat_message(ctx.clone(), msg);
            }
        }
    ));

    text_entry.connect_activate(clone!(
        #[weak] text_entry,
        #[weak] ctx,
        move |_| {
            if text_entry.text().is_empty() { return; }
            idle_add_local_once(clone!(
                #[weak] text_entry,
                move || {
                    text_entry.set_text("");
                }
            ));

            if let Err(e) = on_send_message(ctx.clone(), &text_entry.text()) {
                let msg = format!("Send message error: {}", e.to_string()).to_string();
                add_chat_message(ctx.clone(), msg);
            }
        }
    ));

    send_box.append(&send_btn);

    main_box.append(&send_box);

    let scrolled_window_weak = Downgrade::downgrade(&chat_scrolled);

    idle_add_local({
        let scrolled_window_weak = scrolled_window_weak.clone();
        
        move || {
            if let Some(o) = scrolled_window_weak.upgrade() {
                o.vadjustment().set_value(o.vadjustment().upper() - o.vadjustment().page_size());
            }
            ControlFlow::Break
        }
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("bRAC - Connected to {} as {}", ctx.config(|o| o.host.clone()), &ctx.name()))
        .default_width(500)
        .default_height(500)
        .resizable(false)
        .decorated(true)
        .show_menubar(true)
        .child(&main_box)
        .build();

    window.connect_default_width_notify({
        let scrolled_window_weak = scrolled_window_weak.clone();

        move |_| {
            let scrolled_window_weak = scrolled_window_weak.clone();
            idle_add_local(move || {
                if let Some(o) = scrolled_window_weak.upgrade() {
                    o.vadjustment().set_value(o.vadjustment().upper() - o.vadjustment().page_size());
                }
                ControlFlow::Break
            });
        }
    });

    window.present();

    UiModel {
        chat_scrolled,
        chat_box
    }
}

fn run_recv_loop(ctx: Arc<Context>) {
    thread::spawn(move || {
        loop { 
            if let Err(e) = recv_tick(ctx.clone()) {
                let _ = print_message(ctx.clone(), format!("Print messages error: {}", e.to_string()).to_string());
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

fn setup(ctx: Arc<Context>, ui: UiModel) {
    let (sender, receiver) = channel();

    *ctx.sender.write().unwrap() = Some(Arc::new(sender));

    run_recv_loop(ctx.clone());

    let (tx, rx) = channel();

    GLOBAL.with(|global| {
        *global.borrow_mut() = Some((ui, rx));
    });

    thread::spawn({
        let ctx = ctx.clone();
        move || {
            while let Ok(message) = receiver.recv() {
                let _ = tx.send(message.clone());
                let ctx = ctx.clone();
                glib::source::timeout_add_once(Duration::ZERO, move || {
                    GLOBAL.with(|global| {
                        if let Some((ui, rx)) = &*global.borrow() {
                            let message: String = rx.recv().unwrap();
                            on_add_message(ctx.clone(), &ui, message);
                        }
                    });
                });
            }
        }
    });
}

fn load_css() {
    let is_dark_theme = if let Some(settings) = Settings::default() {
        settings.is_gtk_application_prefer_dark_theme() || settings.gtk_theme_name()
            .map(|o| o.to_lowercase().contains("dark"))
            .unwrap_or_default()
    } else {
        false
    };

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

fn on_add_message(ctx: Arc<Context>, ui: &UiModel, message: String) {
    if message.is_empty() {
        return;
    }

    let hbox = GtkBox::new(Orientation::Horizontal, 2);

    if let Some((date, ip, content, nick)) = parse_message(message.clone()) {
        if let Some(ip) = ip {
            if ctx.config(|o| o.show_other_ip) {
                let ip = Label::builder()
                    .label(ip)
                    .margin_end(10)
                    .halign(Align::Start)
                    .valign(Align::Start)
                    .css_classes(["message-ip"])
                    .selectable(true)
                    .build();

                hbox.append(&ip);
            }
        }

        let date = Label::builder()
            .label(format!("[{date}]"))
            .halign(Align::Start)
            .valign(Align::Start)
            .css_classes(["message-date"])
            .selectable(true)
            .build();

        hbox.append(&date);

        if let Some((name, color)) = nick {
            let name = Label::builder()
                .label(format!("<{name}>"))
                .halign(Align::Start)
                .valign(Align::Start)
                .css_classes(["message-name", &format!("message-name-{}", color)])
                .selectable(true)
                .build();

            hbox.append(&name);
        }

        let content = Label::builder()
            .label(content)
            .halign(Align::Start)
            .valign(Align::Start)
            .css_classes(["message-content"])
            .selectable(true)
            .wrap(true)
            .wrap_mode(WrapMode::Char)
            .build();

        hbox.append(&content);
    } else {
        let content = Label::builder()
            .label(message)
            .halign(Align::Start)
            .valign(Align::Start)
            .css_classes(["message-content"])
            .selectable(true)
            .wrap(true)
            .wrap_mode(WrapMode::Char)
            .build();

        hbox.append(&content);
    }

    ui.chat_box.append(&hbox);

    timeout_add_local_once(Duration::from_millis(1000), move || {
        GLOBAL.with(|global| {
            if let Some((ui, _)) = &*global.borrow() {
                let o = &ui.chat_scrolled;
                o.vadjustment().set_value(o.vadjustment().upper() - o.vadjustment().page_size());
            }
        });
    });
}

pub fn run_main_loop(ctx: Arc<Context>) {
    let application = Application::builder()
        .application_id("ru.themixray.bRAC")
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    application.connect_activate({
        let ctx = ctx.clone();

        move |app| {
            let ui = build_ui(ctx.clone(), app);
            setup(ctx.clone(), ui);
            load_css();
        }
    });

    application.connect_startup({
        let ctx = ctx.clone();

        move |app| {
            build_menu(ctx.clone(), app);
        }
    });

    application.run_with_args::<&str>(&[]);
}