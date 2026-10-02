use std::path::PathBuf;

pub fn windows_download_dir() -> PathBuf {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let downloads = PathBuf::from(userprofile).join("Downloads");
        return downloads;
    }
    crate::platform::common::fallback_download_dir()
}

pub fn windows_config_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("aurora");
    }
    crate::platform::common::fallback_config_dir()
}

pub fn windows_data_dir() -> PathBuf {
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(localappdata).join("aurora");
    }
    crate::platform::common::fallback_data_dir()
}
