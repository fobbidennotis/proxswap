use crate::configuration::IptablesRule;
use std::process::Command;

pub async fn start_redsocks(name: &str) {
    kill_redsocks().await;
    let _ = dbg!(Command::new("redsocks")
        .args(["-c", &format!("./config/redsocks/{}.conf", name)])
        .output()
        .unwrap());
}

async fn kill_redsocks() {
    let _ = Command::new("sudo")
        .args(["killall", "redsocks"])
        .output()
        .unwrap();
}

pub async fn make_iptables_rule(rule: &IptablesRule) -> anyhow::Result<()> {
    let status = dbg!(Command::new("sudo")
        .args([
            "iptables",
            "-t",
            "nat",
            "-A",
            "OUTPUT",
            "-p",
            "tcp",
            "--dport",
            &rule.dport.to_string(),
            "-j",
            &rule.action,
            "--to-port",
            &rule.to_port.to_string()
        ])
        .status()
        .unwrap());

    if !status.success() {
        anyhow::bail!("Couldn't make an iptables rule");
    }

    Ok(())
}
