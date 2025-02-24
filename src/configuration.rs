use crate::bindings::{make_iptables_rule, start_redsocks};
use anyhow::Ok;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Proxy {
    pub proxy_type: String,
    pub url: String,
    pub port: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IptablesRule {
    pub dport: u16,
    pub to_port: u16,
    pub action: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub name: String,
    pub proxies: Vec<Proxy>,
    pub rules: Vec<IptablesRule>,
}

impl Configuration {
    pub async fn new(
        config_name: String,
        proxies: Vec<Proxy>,
        rules: Vec<IptablesRule>,
    ) -> Configuration {
        let conf = if Path::new(&format!("./config/proxswap/{}.json", &config_name)).is_file() {
            Configuration {
                name: config_name,
                proxies: serde_json::from_str("").unwrap(),
                rules: Vec::new(),
            }
        } else {
            let _ = File::create(format!("./config/proxswap/{}.json", &config_name));
            Configuration {
                name: config_name,
                proxies: proxies,
                rules: rules,
            }
        };

        conf.generate_redsocks_config().await.unwrap();

        for rule in conf.rules.iter() {
            let _ = make_iptables_rule(rule).await;
        }

        conf.make_configuration_file().await.unwrap();

        conf
    }

    pub async fn run(&self) {
        start_redsocks(&self.name).await;

        for rule in self.rules.iter() {
            let _ = make_iptables_rule(rule).await;
        }
    }

    async fn make_configuration_file(&self) -> Result<(), anyhow::Error> {
        let mut file = File::create(format!("./config/{}.json", &self.name)).unwrap();
        let json = serde_json::to_string(&self).unwrap();

        let _ = file.write_all(json.as_bytes());

        Ok(())
    }

    async fn generate_redsocks_config(&self) -> Result<(), anyhow::Error> {
        let mut file = File::create(format!("./config/redsocks/{}.conf", &self.name))?;
        let mut proxy_chain: Vec<String> = vec![r#"base {
    log_debug = off;
    log_info = off;
    daemon = on;
    redirector = iptables;
}
"#
        .to_string()]; // Init a vector with base configured
        let mut local_port = 14888; // start port used by the app, the first proxy's local_port in the chain

        for proxy in self.proxies.iter() {
            proxy_chain.push(format!(
                r#"redsocks {{
    local_ip = 127.0.0.1;
    local_port = {};

    type = {};
    ip = {};
    port = {};
}}
"#,
                local_port, proxy.proxy_type, proxy.url, proxy.port
            ));
            local_port += 1;
        }

        println!("{}", proxy_chain.join("\n"));

        let _ = file.write_all(proxy_chain.join("\n").as_bytes());

        Ok(())
    }
}
