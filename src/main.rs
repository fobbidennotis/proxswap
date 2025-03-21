mod configuration;
use crate::configuration::Configuration;
use anyhow::Ok;
use serde_json::from_reader;
use std::fs::{read_dir, File, create_dir_all};
use std::io::BufReader;
mod bindings;
mod tui;
mod paths;
use paths::*;


async fn make_config_directories() -> anyhow::Result<()> {
    create_dir_all(&*REDSOCKS_DIR)?;

    Ok(())
}

async fn init_configurations_dir(dir_path: &str) -> Vec<Configuration> {
    let mut configurations: Vec<Configuration> = Vec::new();

    let matching_files: Vec<String> = read_dir(dir_path)
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("json"))
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect();

    for file in matching_files.iter() {
        configurations.push(init_configuration(file.to_string()).await);
    }

    configurations
}


async fn init_configuration(file_path: String) -> Configuration {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let config: Configuration = from_reader(reader).expect("Failed to parse JSON");

    Configuration::new(config.name, config.proxies, config.rules).await
}

#[tokio::main]
async fn main() {
    make_config_directories().await.unwrap();

    let configurations = init_configurations_dir(&CONFIG_DIR).await;
    let mut app = tui::App::new(configurations);
    app.run().await.expect("Failed to run application");
}
