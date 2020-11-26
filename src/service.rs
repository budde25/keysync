use anyhow::{Context, Result};
use std::env::current_exe;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::util;

const SERVICE_NAME: &str = "keysync.service";

#[derive(PartialEq)]
enum KeysyncService {
    Active,
    Stopped,
    NotInstalled,
}

pub fn check() -> Result<()> {
    let status = get_service_status()?;
    if status == KeysyncService::NotInstalled
        && util::get_confirmation(
            "Systemd service file not installed, would you like it install and enable it now?",
        )?
    {
        // Install and enable
        let mut cmd = Command::new("sudo")
            .arg("keysync")
            .arg("daemon")
            .arg("--install")
            .arg("--enable")
            .spawn()?;
        if cmd.wait()?.success() {
            println!("Sucessfully installed and enabled keysync service");
        }
    }

    if status == KeysyncService::Stopped
        && util::get_confirmation("keysync service not running, would you like to enable it now?")?
    {
        // Enable
        let mut cmd = Command::new("sudo")
            .arg("keysync")
            .arg("daemon")
            .arg("--enable")
            .spawn()?;
        if cmd.wait()?.success() {
            println!("Sucessfully enabled keysync service");
        }
    }
    Ok(())
}

fn get_service_status() -> Result<KeysyncService> {
    let mut cmd = Command::new("systemctl")
        .arg("status")
        .arg(SERVICE_NAME)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Error spawning systemctl, is Systemd installed?")?;
    let code = cmd.wait()?.code().unwrap_or(0);

    match code {
        0 => Ok(KeysyncService::Active),
        1 | 2 | 3 => Ok(KeysyncService::Stopped),
        4 | _ => Ok(KeysyncService::NotInstalled),
    }
}

pub fn enable_service() -> Result<bool> {
    let mut cmd = Command::new("systemctl")
        .arg("enable")
        .arg("--now")
        .arg(SERVICE_NAME)
        .spawn()
        .context("Error spawning systemctl, is Systemd installed?")?;
    Ok(cmd.wait()?.success())
}

pub fn install_service() -> Result<()> {
    let path = PathBuf::from("/usr/lib/systemd/system/").join(SERVICE_NAME);
    let mut file = File::create(path)?;
    let bin = current_exe()?;
    let text = format!(
    "[Unit]\nDescription=The SSH Key Sync service\nAfter=network.target\n[Service]\nExecStart={} daemon\nType=simple\n[Install]\nWantedBy=multi-user.service",
        bin.display()
    );
    file.write_all(text.as_bytes())?;
    Ok(())
}
