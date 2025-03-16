use crate::configuration::IptablesRule;
use std::process::Command;
use crate::paths::*;


pub async fn start_redsocks(name: &str) {
    kill_redsocks().await;
    let _ = Command::new("redsocks")
        .args(["-c", &format!("{}/{}.conf", &*REDSOCKS_DIR, name)])
        .output();
}

pub async fn kill_redsocks() {
    let _ = Command::new("sudo")
        .args(["killall", "redsocks"])
        .output();
}

pub async fn flush_iptables() {
    let _ = Command::new("sudo")
        .args(["iptables", "-t", "nat", "-F"])
        .output();
}

pub async fn deactivate_proxy() {
    kill_redsocks().await;
    flush_iptables().await;
}

pub async fn make_iptables_rule(rule: &IptablesRule) -> anyhow::Result<()> {
    let status = Command::new("sudo")
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
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute iptables command: {}", e))?;

    if !status.status.success() {
        anyhow::bail!("Couldn't make an iptables rule");
    }

    Ok(())
}
