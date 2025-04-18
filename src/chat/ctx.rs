use std::sync::{atomic::{AtomicUsize, Ordering}, mpsc::Sender, Arc, RwLock};

use super::config::Config;

pub struct Context {
    pub registered: Arc<RwLock<Option<String>>>,
    pub config: Arc<RwLock<Config>>,
    pub sender: Arc<RwLock<Option<Arc<Sender<String>>>>>,
    pub messages: RwLock<Vec<String>>,
    pub packet_size: AtomicUsize,
    pub name: String
}

impl Context {
    pub fn new(config: &Config) -> Context {
        Context {
            registered: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config.clone())),
            sender: Arc::new(RwLock::new(None)),
            messages: RwLock::new(Vec::new()),
            packet_size: AtomicUsize::default(),
            name: config.name.clone().expect("not implemented"), // TODO: ask for name
        }
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

    pub fn update(&self, max_length: usize, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        let mut messages = messages;
        if messages.len() > max_length {
            messages.drain(max_length..);
        }
        *self.messages.write().unwrap() = messages;
    }

    pub fn append_and_store(&self, max_length: usize, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        self.append(max_length, messages);
    }

    pub fn append(&self, max_length: usize, messages: Vec<String>) {
        self.messages.write().unwrap().append(&mut messages.clone());
        if self.messages.read().unwrap().len() > max_length {
            self.messages.write().unwrap().drain(max_length..);
        }
    }
}

#[macro_export]
macro_rules! connect_rac {
    ($ctx:ident) => { &mut connect(&$ctx.config(|o| o.host.clone()), $ctx.config(|o| o.ssl_enabled))? };
}