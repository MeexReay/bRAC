use std::sync::{atomic::Ordering, Arc};
use std::thread;
use std::time::{Duration, SystemTime};

use chrono::Local;

use adw::gdk::{Cursor, Display};
use adw::gio::MemoryInputStream;
use adw::glib::clone;
use adw::glib::{self, source::timeout_add_local_once, timeout_add_local, ControlFlow};
use adw::prelude::*;
use adw::Application;
use libadwaita::gdk::{BUTTON_PRIMARY, BUTTON_SECONDARY};
use libadwaita::gtk::{GestureLongPress, MenuButton, Popover};
use libadwaita::{self as adw, Avatar, HeaderBar, ToolbarView};

use adw::gtk;
use gtk::gdk_pixbuf::PixbufAnimation;
use gtk::pango::WrapMode;
use gtk::{
    Align, Box as GtkBox, Button, Calendar, Entry, Fixed, GestureClick, Justification, Label,
    ListBox, Orientation, Overlay, Picture, ScrolledWindow,
};

use crate::chat::{
    config::get_config_path, ctx::Context, on_send_message, parse_message, SERVER_LIST,
};

use super::widgets::CustomLayout;
use super::{
    add_chat_messages, build_menu, get_avatar_id, get_message_sign, load_pixbuf, send_notification,
    try_save_config, update_window_title, UiModel,
};

pub fn get_message_box(
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

fn open_avatar_popup(avatar: String, avatar_picture: &Avatar) {
    let display = Display::default().unwrap();
    let clipboard = display.clipboard();

    let popover = Popover::new();

    let button = Button::with_label("Copy Link");
    button.connect_clicked(clone!(
        #[weak]
        clipboard,
        #[weak]
        popover,
        #[strong]
        avatar,
        move |_| {
            clipboard.set_text(avatar.as_str());
            popover.popdown();
        }
    ));

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();
    vbox.append(&button);

    popover.set_child(Some(&vbox));
    popover.set_parent(avatar_picture);
    popover.popup();
}

pub fn get_new_message_box(
    ctx: Arc<Context>,
    ui: &UiModel,
    message: String,
    notify: bool,
    formatting_enabled: bool,
) -> Overlay {
    // TODO: softcode these colors

    let (ip_color, date_color, text_color) = if ui.is_dark_theme {
        ("#494949", "#929292", "#FFFFFF")
    } else {
        ("#585858", "#292929", "#000000")
    };

    let latest_sign = ui.latest_sign.load(Ordering::SeqCst);

    let (date, ip, content, name, color, avatar, avatar_id) =
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
                avatar.clone(),
                avatar.map(|o| get_avatar_id(&o)).unwrap_or_default(),
            )
        } else {
            (
                Local::now().format("%d.%m.%Y %H:%M").to_string(),
                None,
                message,
                "System".to_string(),
                "#DDDDDD".to_string(),
                None,
                0,
            )
        };

    if notify && !ui.window.is_active() {
        if ctx.config(|o| o.chunked_enabled) {
            send_notification(
                ctx.clone(),
                ui,
                &if name == *"System" {
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
            .size(32)
            .build();

        avatar_picture.set_vexpand(false);
        avatar_picture.set_hexpand(false);
        avatar_picture.set_valign(Align::Start);
        avatar_picture.set_halign(Align::Start);

        if let Some(avatar) = avatar {
            let long_gesture = GestureLongPress::builder().button(BUTTON_PRIMARY).build();

            long_gesture.connect_pressed(clone!(
                #[weak]
                avatar_picture,
                #[strong]
                avatar,
                move |_, x, y| {
                    if x < 32.0 && y > 4.0 && y < 32.0 {
                        open_avatar_popup(avatar.clone(), &avatar_picture);
                    }
                }
            ));

            overlay.add_controller(long_gesture);

            let short_gesture = GestureClick::builder().button(BUTTON_SECONDARY).build();

            short_gesture.connect_released(clone!(
                #[weak]
                avatar_picture,
                #[strong]
                avatar,
                move |_, _, x, y| {
                    if x < 32.0 && y > 4.0 && y < 32.0 {
                        open_avatar_popup(avatar.clone(), &avatar_picture);
                    }
                }
            ));

            overlay.add_controller(short_gesture);
        }

        if avatar_id != 0 {
            let mut lock = ui.avatars.lock().unwrap();

            if let Some(pics) = lock.get_mut(&avatar_id) {
                pics.push(avatar_picture.clone());
            } else {
                lock.insert(avatar_id, vec![avatar_picture.clone()]);
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

    vbox.append(
        &Label::builder()
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
            .build(),
    );

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

/// header, page_box, chat_box, chat_scrolled
pub fn build_page(
    ctx: Arc<Context>,
    app: &Application,
) -> (HeaderBar, GtkBox, GtkBox, ScrolledWindow) {
    let page_box = GtkBox::new(Orientation::Vertical, 5);
    page_box.set_css_classes(&["page-box"]);

    let toolbar = ToolbarView::new();

    let header = HeaderBar::new();

    header.pack_end(
        &MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&build_menu(ctx.clone(), &app))
            .build(),
    );

    toolbar.set_content(Some(&header));

    page_box.append(&toolbar);

    page_box.append(&build_widget_box(ctx.clone(), app));

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

    let layout = CustomLayout::default();

    layout.connect_local("size-changed", false, {
        let chat_scrolled = chat_scrolled.downgrade();
        move |_| {
            if let Some(chat_scrolled) = chat_scrolled.upgrade() {
                let value =
                    chat_scrolled.vadjustment().upper() - chat_scrolled.vadjustment().page_size();
                chat_scrolled.vadjustment().set_value(value);
            }
            return None;
        }
    });

    page_box.set_layout_manager(Some(layout));

    timeout_add_local_once(
        Duration::ZERO,
        clone!(
            #[weak]
            chat_scrolled,
            move || {
                let value =
                    chat_scrolled.vadjustment().upper() - chat_scrolled.vadjustment().page_size();
                chat_scrolled.vadjustment().set_value(value);
            }
        ),
    );

    page_box.append(&chat_scrolled);

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
        .css_classes(["send-text", "suggested-action"])
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

    page_box.append(&send_box);

    (header, page_box, chat_box, chat_scrolled)
}

fn build_widget_box(ctx: Arc<Context>, _app: &Application) -> Overlay {
    let widget_box_overlay = Overlay::new();

    let widget_box = GtkBox::new(Orientation::Horizontal, 5);
    widget_box.set_css_classes(&["widget-box"]);

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

    widget_box_overlay
}
