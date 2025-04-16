use std::sync::{Arc, RwLock};
use std::time::Duration;

use colored::{Color, Colorize};
use gtk4::gdk::{Cursor, Display};
use gtk4::gdk_pixbuf::PixbufLoader;
use gtk4::glib::clone::Downgrade;
use gtk4::glib::{idle_add_local, idle_add_local_once, ControlFlow, source::timeout_add_local_once};
use gtk4::{glib, glib::clone, Align, Box as GtkBox, Label, ScrolledWindow};
use gtk4::{CssProvider, Entry, Orientation, Overlay, Picture};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::error::Error;
use std::thread;
use std::cell::RefCell;

use crate::config::Context;
use crate::proto::{connect, read_messages};

use super::{format_message, on_send_message, parse_message, set_chat, ChatStorage};


pub struct ChatContext {
    pub messages: Arc<ChatStorage>, 
    pub registered: Arc<RwLock<Option<String>>>,
    pub sender: Sender<String>
}

struct UiModel {
    chat_box: GtkBox,
    chat_scrolled: ScrolledWindow
}

thread_local!(
    static GLOBAL: RefCell<Option<(UiModel, Receiver<String>)>> = RefCell::new(None);
);

pub fn add_chat_message(ctx: Arc<Context>, message: String) {
    let _ = ctx.chat().sender.send(message);
}

pub fn print_message(ctx: Arc<Context>, message: String) -> Result<(), Box<dyn Error>> {
    ctx.chat().messages.append(ctx.max_messages, vec![message.clone()]);
    add_chat_message(ctx.clone(), message);
    Ok(())
}

pub fn recv_tick(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    match read_messages(
        &mut connect(&ctx.host, ctx.enable_ssl)?, 
        ctx.max_messages, 
        ctx.chat().messages.packet_size(), 
        !ctx.enable_ssl,
        ctx.enable_chunked
    ) {
        Ok(Some((messages, size))) => {
            let messages: Vec<String> = if ctx.disable_formatting {
                messages 
            } else {
                messages.into_iter().flat_map(|o| format_message(ctx.enable_ip_viewing, o)).collect()
            };

            if ctx.enable_chunked {
                ctx.chat().messages.append_and_store(ctx.max_messages, messages.clone(), size);
                for msg in messages {
                    add_chat_message(ctx.clone(), msg.clone());
                }
            } else {
                ctx.chat().messages.update(ctx.max_messages, messages.clone(), size);
                for msg in messages {
                    add_chat_message(ctx.clone(), msg.clone());
                }
            }
        },
        Err(e) => {
            let msg = format!("Read messages error: {}", e.to_string()).bright_red().to_string();
            ctx.chat().messages.append(ctx.max_messages, vec![msg.clone()]);
            add_chat_message(ctx.clone(), msg.clone());
        }
        _ => {}
    }
    thread::sleep(Duration::from_millis(ctx.update_time as u64));
    Ok(())
}

fn build_ui(ctx: Arc<Context>, app: &Application) {
    let main_box = GtkBox::new(Orientation::Vertical, 5);

    main_box.set_margin_bottom(5);
    main_box.set_margin_end(5);
    main_box.set_margin_start(5);
    main_box.set_margin_top(5);

    let chat_box = GtkBox::new(Orientation::Vertical, 2);

    let chat_scrolled = ScrolledWindow::builder()
        .child(&chat_box)
        .vexpand(true)
        .hexpand(true)
        .propagate_natural_height(true)
        .build();

    main_box.append(&chat_scrolled);

    let send_box = GtkBox::new(Orientation::Horizontal, 5);

    let text_entry = Entry::builder()
        .placeholder_text("Message")
        .hexpand(true)
        .build();

    send_box.append(&text_entry);

    let send_btn = Button::builder()
        .label("Send")
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
                let msg = format!("Send message error: {}", e.to_string()).bright_red().to_string();
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
                let msg = format!("Send message error: {}", e.to_string()).bright_red().to_string();
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

    let overlay = Overlay::new();

    overlay.set_child(Some(&main_box));

    let bytes = include_bytes!("../../brac_logo.png");
    let loader = PixbufLoader::new();
    loader.write(bytes).unwrap();
    loader.close().unwrap();
    let pixbuf = loader.pixbuf().unwrap();

    let logo = Picture::for_pixbuf(&pixbuf);
    logo.set_size_request(500, 189);
    logo.set_can_target(false);
    logo.set_can_focus(false);
    logo.set_halign(Align::End);
    logo.set_valign(Align::Start);

    overlay.add_overlay(&logo);
    
    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("bRAC - Connected to {} as {}", &ctx.host, &ctx.name))
        .default_width(500)
        .default_height(500)
        .resizable(false)
        .decorated(true)
        .child(&overlay)
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

    window.show();

    let ui = UiModel {
        chat_scrolled,
        chat_box
    };

    setup(ctx.clone(), ui);
    load_css();
}

fn setup(ctx: Arc<Context>, ui: UiModel) {
    let (sender, receiver) = channel();

    set_chat(ctx.clone(), ChatContext {
        messages: Arc::new(ChatStorage::new()), 
        registered: Arc::new(RwLock::new(None)),
        sender
    });

    thread::spawn({
        let ctx = ctx.clone();

        move || {
            loop { 
                if let Err(e) = recv_tick(ctx.clone()) {
                    let _ = print_message(ctx.clone(), format!("Print messages error: {}", e.to_string()).bright_red().to_string());
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });

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
    let provider = CssProvider::new();
    provider.load_from_data("

        * {
            border-radius: 0;
        }

        .message-content {
            color: #000000;
        }

        .message-date {
            color: #555555;
        }

        .message-ip {
            color: #777777;
        }

        .message-name {
            font-weight: bold;
        }

        .message-name-black {
            color: #2E2E2E; /* Темный черный */
        }

        .message-name-red {
            color: #8B0000; /* Темный красный */
        }

        .message-name-green {
            color: #006400; /* Темный зеленый */
        }

        .message-name-yellow {
            color: #8B8B00; /* Темный желтый */
        }

        .message-name-blue {
            color: #00008B; /* Темный синий */
        }

        .message-name-magenta {
            color: #8B008B; /* Темный пурпурный */
        }

        .message-name-cyan {
            color: #008B8B; /* Темный бирюзовый */
        }

        .message-name-white {
            color: #A9A9A9; /* Темный белый */
        }

        .message-name-bright-black {
            color: #555555; /* Яркий черный */
        }

        .message-name-bright-red {
            color: #FF0000; /* Яркий красный */
        }

        .message-name-bright-green {
            color: #00FF00; /* Яркий зеленый */
        }

        .message-name-bright-yellow {
            color: #FFFF00; /* Яркий желтый */
        }

        .message-name-bright-blue {
            color: #0000FF; /* Яркий синий */
        }

        .message-name-bright-magenta {
            color: #FF00FF; /* Яркий пурпурный */
        }

        .message-name-bright-cyan {
            color: #00FFFF; /* Яркий бирюзовый */
        }

        .message-name-bright-white {
            color: #FFFFFF; /* Яркий белый */
        }

    ");

    gtk4::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(false);
    }
}

fn on_add_message(ctx: Arc<Context>, ui: &UiModel, message: String) {
    let hbox = GtkBox::new(Orientation::Horizontal, 2);

    if let Some((date, ip, content, nick)) = parse_message(message.clone()) {
        if let Some(ip) = ip {
            if ctx.enable_ip_viewing {
                let ip = Label::builder()
                    .label(ip)
                    .margin_end(10)
                    .halign(Align::Start)
                    .css_classes(["message-ip"])
                    .build();

                hbox.append(&ip);
            }
        }

        let date = Label::builder()
            .label(format!("[{date}]"))
            .halign(Align::Start)
            .css_classes(["message-date"])
            .build();

        hbox.append(&date);

        if let Some((name, color)) = nick {
            let color = match color {
                Color::Black => "black",
                Color::Red => "red",
                Color::Green => "green",
                Color::Yellow => "yellow",
                Color::Blue => "blue",
                Color::Magenta => "magenta",
                Color::Cyan => "cyan",
                Color::White => "white",
                Color::BrightBlack => "bright-black",
                Color::BrightRed => "bright-red",
                Color::BrightGreen => "bright-green",
                Color::BrightYellow => "bright-yellow",
                Color::BrightBlue => "bright-blue",
                Color::BrightMagenta => "bright-magenta",
                Color::BrightCyan => "bright-cyan",
                Color::BrightWhite => "bright-white",
                _ => "unknown"
            };

            let name = Label::builder()
                .label(format!("<{name}>"))
                .halign(Align::Start)
                .css_classes(["message-name", &format!("message-name-{}", color)])
                .build();

            hbox.append(&name);
        }

        let content = Label::builder()
            .label(content)
            .halign(Align::Start)
            .css_classes(["message-content"])
            .build();

        hbox.append(&content);
    } else {
        let content = Label::builder()
            .label(message)
            .halign(Align::Start)
            .css_classes(["message-content"])
            .build();

        hbox.append(&content);
    }

    ui.chat_box.append(&hbox);

    timeout_add_local_once(Duration::from_millis(100), move || {
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
        .build();

    application.connect_activate({
        let ctx = ctx.clone();

        move |app| {
            build_ui(ctx.clone(), app);
        }
    });

    application.run();
}