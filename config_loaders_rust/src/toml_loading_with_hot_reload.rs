use anyhow::Result;
use lazy_static::lazy_static;
use notify::Watcher;
use parking_lot::Mutex;
use serde_derive::Deserialize;
use tokio::time::{self, Duration, Instant};
use toml;

use std::{fs, sync::Arc, thread};

lazy_static! {
    pub static ref conf: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::new("./config.toml")));
    static ref running_threads: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    static ref last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub global: Global,
    path: String,
}

#[derive(Deserialize, Debug)]
pub struct InnerConfig {
    pub global: Global,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Global {
    pub cool_string: String,
}

fn open_config(path: &str) -> InnerConfig {
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            panic!("Couldn't open config file due to {e}");
        }
    };

    match toml::from_str(&contents) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{e}");
            panic!("Unable to load data from {}", path);
        }
    }
}

impl Config {
    pub fn new(path: &str) -> Self {
        let data = open_config(path);

        Config {
            global: data.global,
            path: path.to_string(),
        }
    }

    pub fn start_watchdog(config: Arc<Mutex<Self>>) {
        tokio::spawn(Self::watchdog_thread(config));
    }

    pub fn config_reload() {
        let mut config_lock = conf.lock();
        let data = open_config(&config_lock.path);

        config_lock.global = data.global;
    }

    fn do_stuff(event: notify::Event) {
        if let notify::EventKind::Modify(_) = event.kind {
            let mut time_lock = last_event.lock();
            let now = Instant::now();

            // just try to catch the first event and then return everytime
            if now.duration_since(*time_lock).as_millis() >= 25 {
                // introduce small delay to cope with the many different file saving things
                thread::sleep(Duration::from_millis(50));
                Self::config_reload();
            }

            *time_lock = Instant::now();
        }
    }

    async fn watchdog_thread(config: Arc<Mutex<Self>>) -> Result<()> {
        let mut interval = time::interval(Duration::from_secs(1));
        let path = config.clone().lock().path.clone();

        // trigger the first event, basically runs the function inside the lazy_static
        drop(last_event.lock());

        //let mut active_threads = running_threads.lock();
        // push the current thread path to the running_threads global, so we can prevent starting
        // the same thread twice
        //running_threads.lock().push(config.lock().path.clone());

        //if active_threads.contains(&path) {
        //    return Ok(());
        //}

        let mut watcher = notify::recommended_watcher(move |r| match r {
            Ok(event) => Self::do_stuff(event),
            Err(_) => todo!(),
        })?;

        watcher.watch(
            std::path::Path::new(&path),
            notify::RecursiveMode::NonRecursive,
        )?;

        // keep the thread alive
        loop {
            interval.tick().await;
        }
    }
}
