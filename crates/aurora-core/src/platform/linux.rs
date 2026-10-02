use std::path::PathBuf;

pub fn linux_download_dir() -> PathBuf {
    if let Ok(xdg_download) = std::env::var("XDG_DOWNLOAD_DIR") {
        if !xdg_download.trim().is_empty() {
            return PathBuf::from(xdg_download);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let downloads = PathBuf::from(home).join("Downloads");
        return downloads;
    }
    crate::platform::common::fallback_download_dir()
}

pub fn linux_config_dir() -> PathBuf {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg_config.trim().is_empty() {
            return PathBuf::from(xdg_config).join("aurora");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join("aurora");
    }
    crate::platform::common::fallback_config_dir()
}

pub fn linux_data_dir() -> PathBuf {
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        if !xdg_data.trim().is_empty() {
            return PathBuf::from(xdg_data).join("aurora");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local").join("share").join("aurora");
    }
    crate::platform::common::fallback_data_dir()
}
