mod configuration;
use crate::configuration::Configuration;
use std::fs::{read_dir, File};
use std::io::BufReader;
use configuration::{IptablesRule, Proxy};
use serde_json::from_reader;
mod bindings;



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
    let mut current_configurations = dbg!(init_configurations_dir("./config/").await);

    for conf in current_configurations.iter() {
        dbg!(conf);
    }

    current_configurations.push(Configuration::new("Yet Another Configuration".to_string(),
            vec![
                Proxy { proxy_type: "socks5".to_string(), url: "185.182.111.54".to_string(), port: 1488 }
            ],
            vec![
                IptablesRule { dport: 80, to_port: 443, action: "REDIRECT".to_string() }
            ]
            ).await);

    current_configurations[0].run().await;
}
