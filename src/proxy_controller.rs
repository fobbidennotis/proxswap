use std::fmt::write;
use std::fs::File;
use std::io::prelude::*;

pub struct Proxy {
    pub proxy_type: String,
    pub url: String,
    pub port: u32
}

pub async fn generate_proxy_file(proxies: Vec<Proxy>) -> std::io::Result<()> {
    let mut file = File::create("./redsocks_conf/redsocks.conf")?;
    let mut proxy_chain: Vec<String> = vec![
r#"base {
    log_debug = off;
    log_info = off;
    daemon = on;
    redirector = iptables;
}
"#.to_string()]; // Init a vector with base configured
    let mut local_port = 14888; // start port used by the app, the first proxy's local_port in the chain


    for proxy in proxies.iter() {
        proxy_chain.push(
format!(r#"redsocks {{
    local_ip = 127.0.0.1;
    local_port = {};

    type = {};
    ip = {};
    port = {};
}}
"#, local_port, proxy.proxy_type, proxy.url, proxy.port));
        local_port += 1;
    }

    println!("{}", proxy_chain.join("\n"));

    let _ = file.write_all(proxy_chain.join("\n").as_bytes());

    Ok(())
}
