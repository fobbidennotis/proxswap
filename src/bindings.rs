use std::process::Command;


pub async fn start_redsocks(config_path: String) {
    let _ = dbg!(Command::new("redsocks")
        .args(["-c", &config_path])
        .output()
        .unwrap());
}

pub async fn make_iptables_rule(dport: u16, action:&str, to_port: u16) {
    let _ = dbg!(Command::new("sudo")
        .args([
            "iptables",
            "-t",
            "nat",
            "-A",
            "OUTPUT",
            "-p",
            "tcp",
            "--dport",
            &dport.to_string(),
            "-j",
            action,
            "--to-port",
            &to_port.to_string()
        ])
        .output()
        .unwrap());
}
