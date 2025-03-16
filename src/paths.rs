use std::env;
use once_cell::sync::Lazy;

pub static CONFIG_DIR: Lazy<String> = Lazy::new(|| {
    format!("{}/.config/proxswap", env::var("HOME").expect("Failed to get HOME directory"))
});

pub static REDSOCKS_DIR: Lazy<String> = Lazy::new(|| {
    format!("{}/redsocks", *CONFIG_DIR)
});
