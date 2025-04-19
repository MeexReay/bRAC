use std::sync::{atomic::{AtomicUsize, Ordering}, mpsc::Sender, Arc, RwLock};

use rand::random;

use super::config::Config;

pub struct Context {
    pub registered: RwLock<Option<String>>,
    pub config: RwLock<Config>,
    pub sender: RwLock<Option<Arc<Sender<String>>>>,
    pub messages: RwLock<Vec<String>>,
    pub packet_size: AtomicUsize,
    pub name: RwLock<String>
}

impl Context {
    pub fn new(config: &Config) -> Context {
        Context {
            registered: RwLock::new(None),
            config: RwLock::new(config.clone()),
            sender: RwLock::new(None),
            messages: RwLock::new(Vec::new()),
            packet_size: AtomicUsize::default(),
            name: RwLock::new(config.name.clone().unwrap_or_else(|| format!("Anon#{:X}", random::<u16>()))),
        }
    }

    pub fn name(&self) -> String {
        self.name.read().unwrap().clone()
    }

    pub fn set_config(&self, config: &Config) {
        *self.config.write().unwrap() = config.clone();
        *self.name.write().unwrap() = config.name.clone().unwrap_or_else(|| format!("Anon#{:X}", random::<u16>()));
        *self.registered.write().unwrap() = None;
        *self.messages.write().unwrap() = Vec::new();
        self.packet_size.store(0, Ordering::SeqCst);
    }

    pub fn config<T>(&self, map: fn (&Config) -> T) -> T {
        map(&self.config.read().unwrap())
    }

    pub fn packet_size(&self) -> usize {
        self.packet_size.load(Ordering::SeqCst)
    }

    pub fn messages(&self) -> Vec<String> {
        self.messages.read().unwrap().clone()
    }

    pub fn put_messages_packet(&self, max_length: usize, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        let mut messages = messages;
        if messages.len() > max_length {
            messages.drain(max_length..);
        }
        *self.messages.write().unwrap() = messages;
    }

    pub fn add_messages_packet(&self, max_length: usize, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        self.add_message(max_length, messages);
    }

    pub fn add_message(&self, max_length: usize, messages: Vec<String>) {
        self.messages.write().unwrap().append(&mut messages.clone());
        if self.messages.read().unwrap().len() > max_length {
            self.messages.write().unwrap().drain(max_length..);
        }
    }
}

#[macro_export]
macro_rules! connect_rac {
    ($ctx:ident) => { 
        &mut connect(
            &$ctx.config(|o| o.host.clone()), 
            $ctx.config(|o| o.ssl_enabled), 
            $ctx.config(|o| o.proxy.clone())
        )? 
    };
}