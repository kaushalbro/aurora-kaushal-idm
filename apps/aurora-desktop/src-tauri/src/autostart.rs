use std::path::PathBuf;

fn get_exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "aurora-desktop".to_string())
}

#[cfg(target_os = "linux")]
fn get_linux_autostart_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|c| c.join("autostart").join("aurora-desktop.desktop"))
}

#[cfg(not(target_os = "linux"))]
fn get_linux_autostart_path() -> Option<PathBuf> {
    None
}

pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Some(path) = get_linux_autostart_path() {
            return path.exists();
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "AURORA Kaushal IDM",
            ])
            .output();
        if let Ok(out) = output {
            return out.status.success();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let plist = PathBuf::from(home)
                .join("Library/LaunchAgents/com.aurora.kaushal.idm.plist");
            return plist.exists();
        }
    }

    false
}

pub fn set_autostart(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if let Some(path) = get_linux_autostart_path() {
            if enable {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let exe = get_exe_path();
                let desktop_entry = format!(
                    "[Desktop Entry]\n\
                    Type=Application\n\
                    Version=1.0\n\
                    Name=AURORA Kaushal IDM\n\
                    Comment=Ultra-High Performance Internet Download Manager\n\
                    Exec=\"{}\" --autostart\n\
                    Icon=aurora-desktop\n\
                    Terminal=false\n\
                    Categories=Network;FileTransfer;Utility;\n\
                    X-GNOME-Autostart-enabled=true\n",
                    exe
                );
                std::fs::write(&path, desktop_entry).map_err(|e| e.to_string())?;
            } else if path.exists() {
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
            return Ok(());
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let exe = get_exe_path();
        if enable {
            let val = format!("\"{}\" --autostart", exe);
            let status = Command::new("reg")
                .args(&[
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "AURORA Kaushal IDM",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &val,
                    "/f",
                ])
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("Failed to set Windows autostart registry key".to_string());
            }
        } else {
            let _ = Command::new("reg")
                .args(&[
                    "delete",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "AURORA Kaushal IDM",
                    "/f",
                ])
                .status();
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let plist_path = PathBuf::from(home)
                .join("Library/LaunchAgents/com.aurora.kaushal.idm.plist");
            if enable {
                if let Some(parent) = plist_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let exe = get_exe_path();
                let plist_content = format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                    <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
                    <plist version=\"1.0\">\n\
                    <dict>\n\
                        <key>Label</key>\n\
                        <string>com.aurora.kaushal.idm</string>\n\
                        <key>ProgramArguments</key>\n\
                        <array>\n\
                            <string>{}</string>\n\
                            <string>--autostart</string>\n\
                        </array>\n\
                        <key>RunAtLoad</key>\n\
                        <true/>\n\
                    </dict>\n\
                    </plist>",
                    exe
                );
                std::fs::write(&plist_path, plist_content).map_err(|e| e.to_string())?;
            } else if plist_path.exists() {
                let _ = std::fs::remove_file(&plist_path);
            }
            return Ok(());
        }
    }

    Ok(())
}
