mod proxy_controller;
use proxy_controller::generate_proxy_file;
use proxy_controller::Proxy;
use bindings::*;
mod bindings;


#[tokio::main]
async fn main() {
    let _ = generate_proxy_file(vec![
        Proxy {
            proxy_type: "socks5".to_string(),
            url: "185.182.111.54".to_string(),
            port: 1488
        },
    ]).await;

    start_redsocks("./redsocks_conf/redsocks.conf".to_string()).await;
    make_iptables_rule(80, "REDIRECT", 14888).await;
    make_iptables_rule(443, "REDIRECT", 14888).await;
}
