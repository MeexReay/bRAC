use std::sync::Arc;

use adw::gdk::Display;
use adw::glib::clone;
use adw::glib::{self};
use adw::prelude::*;
use adw::Application;
use libadwaita::gtk::Adjustment;
use libadwaita::{
    self as adw, ActionRow, ButtonRow, EntryRow, PreferencesDialog, PreferencesGroup,
    PreferencesPage, SpinRow, SwitchRow,
};

use adw::gtk;
use gtk::Button;

use crate::chat::{
    config::{get_config_path, Config},
    ctx::Context,
};

use super::{try_save_config, update_window_title};

pub fn open_settings(ctx: Arc<Context>, app: &Application) {
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
        .title("Avatar Link")
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
        .adjustment(
            &Adjustment::builder()
                .lower(1.0)
                .upper(1048576.0)
                .page_increment(10.0)
                .step_increment(10.0)
                .value(ctx.config(|o| o.max_messages) as f64)
                .build(),
        )
        .build();

    group.add(&messages_limit);

    // Update interval preference

    let update_interval = SpinRow::builder()
        .title("Update interval")
        .subtitle("In milliseconds")
        .adjustment(
            &Adjustment::builder()
                .lower(10.0)
                .upper(1048576.0)
                .page_increment(10.0)
                .step_increment(10.0)
                .value(ctx.config(|o| o.update_time) as f64)
                .build(),
        )
        .build();

    group.add(&update_interval);

    // Update interval OOF preference

    let update_interval_oof = SpinRow::builder()
        .title("Update interval when unfocused")
        .subtitle("In milliseconds")
        .adjustment(
            &Adjustment::builder()
                .lower(10.0)
                .upper(1048576.0)
                .page_increment(10.0)
                .step_increment(10.0)
                .value(ctx.config(|o| o.oof_update_time) as f64)
                .build(),
        )
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

    // config_path_copy.set_css_classes(&["circular"]);
    config_path_copy.set_margin_top(10);
    config_path_copy.set_margin_bottom(10);
    config_path_copy.connect_clicked(clone!(
        #[weak]
        clipboard,
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

    let reset_button = ButtonRow::builder().title("Reset all").build();

    reset_button.connect_activated(clone!(
        #[weak]
        ctx,
        #[weak]
        app,
        #[weak]
        dialog,
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
        .adjustment(
            &Adjustment::builder()
                .lower(0.0)
                .upper(1074790400.0)
                .page_increment(1024.0)
                .step_increment(1024.0)
                .value(ctx.config(|o| o.max_avatar_size) as f64)
                .build(),
        )
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
        .adjustment(
            &Adjustment::builder()
                .lower(0.0)
                .upper(200.0)
                .page_increment(10.0)
                .step_increment(10.0)
                .value(ctx.config(|o| o.konata_size) as f64)
                .build(),
        )
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
        let old_config = ctx.config.read().unwrap().clone();

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
            servers: old_config.servers,
        };
        ctx.set_config(&config);
        try_save_config(get_config_path(), &config);
        update_window_title(ctx.clone());
    });

    dialog.present(app.active_window().as_ref());
}
