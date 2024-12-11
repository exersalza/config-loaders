use config_loaders_rust::toml_loading_with_hot_reload::{conf, Config};

fn main() {
    // just unlazy the lazy conf
    drop(conf.lock());

    {
        Config::start_watchdog(conf.clone());
    }
}
