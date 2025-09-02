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

use clap::crate_version;

use libadwaita::gdk::Texture;
use libadwaita::gtk::gdk_pixbuf::InterpType;
use libadwaita::gtk::MenuButton;
use libadwaita::{
    self as adw, Avatar, HeaderBar
};
use adw::gdk::Display;
use adw::gio::{ActionEntry, ApplicationFlags, Menu};
use adw::glib::clone;
use adw::glib::{
    self, source::timeout_add_local_once, timeout_add_once,
};
use adw::prelude::*;
use adw::{Application, ApplicationWindow};

use adw::gtk;
use gtk::gdk_pixbuf::{Pixbuf, PixbufLoader};
use gtk::{
    Box as GtkBox,
    CssProvider,
    Orientation, ScrolledWindow, Settings,
};

use crate::chat::grab_avatar;

use super::{
    config::{
        save_config, Config,
    },
    ctx::Context, print_message, recv_tick, sanitize_message,
};

mod preferences;
mod page;
mod widgets;

use page::*;
use preferences::*;

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
                        .license(glib::markup_escape_text(include_str!("../../../LICENSE")))
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
    
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    
    let header = HeaderBar::new();

    header.pack_end(&MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&build_menu(ctx.clone(), &app))
        .build());
    
    main_box.append(&header);

    let (page_box, chat_box, chat_scrolled) = build_page_box(ctx.clone(), app);
    
    main_box.append(&page_box);

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
        .content(&main_box)
        .build();

    // window.connect_default_width_notify(clone!(
    //     #[weak] chat_scrolled,
    //     move |_| {
    //         timeout_add_local_once(Duration::ZERO, clone!(
    //             #[weak] chat_scrolled,
    //             move || {
    //                 let value = chat_scrolled.vadjustment().upper() - chat_scrolled.vadjustment().page_size();
    //                 chat_scrolled.vadjustment().set_value(value);
    //             }
    //         ));
    //     }
    // ));
    
    // window.connect_default_height_notify(clone!(
    //     #[weak] chat_scrolled,
    //     move |_| {
    //         timeout_add_local_once(Duration::ZERO, clone!(
    //             #[weak] chat_scrolled,
    //             move || {
    //                 let value = chat_scrolled.vadjustment().upper() - chat_scrolled.vadjustment().page_size();
    //                 chat_scrolled.vadjustment().set_value(value);
    //             }
    //         ));
    //     }
    // ));

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

                                let Some(avatar) = load_avatar(&avatar_url, ctx.config(|o| o.proxy.clone()), ctx.config(|o| o.max_avatar_size as usize)) else { println!("cant load avatar: {avatar_url} request error"); continue };
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

fn get_avatar_id(url: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(url.as_bytes());
    hasher.finish()
}

fn load_avatar(url: &str, proxy: Option<String>, response_limit: usize) -> Option<Vec<u8>> {
    let client = if let Some(proxy) = proxy {
        let proxy = if proxy.starts_with("socks5://") {
            proxy
        } else {
            format!("socks5://{proxy}")
        };
        
        reqwest::blocking::Client::builder()
            .proxy(reqwest::Proxy::all(&proxy).ok()?)
            .build().ok()?
    } else {
        reqwest::blocking::Client::new()
    };
    
    client.get(url).send().ok()
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
