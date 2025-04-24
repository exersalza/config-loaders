// Config
/// Note this macro only works if there's a module called `config` in your src/
#[macro_export]
#[allow(clippy::crate_in_macro_def)] // we have to use `crate` here bc we want to use the binary crate not this
                                     // library
macro_rules! config {
    () => {
        crate::config::conf.lock()
    };
}

/// open the config file and return the contents of it parsed to the struct
#[macro_export]
macro_rules! open_config {
    ($path:expr_2021) => {{
        let _: &str = $path;
        use std::fs;
        use toml;

        let contents = match fs::read_to_string($path) {
            Ok(c) => c,
            Err(e) => {
                panic!("Couldn't open config file due to {e}");
            }
        };

        match toml::from_str(&contents) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("{e}");
                panic!("Unable to load data from {}", $path);
            }
        }
    }};
}
