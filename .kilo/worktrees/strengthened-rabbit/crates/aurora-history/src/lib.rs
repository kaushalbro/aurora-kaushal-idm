pub mod db;
pub mod host_profile;

pub use db::{DownloadRecord, HistoryDatabase};
pub use host_profile::HostProfile;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    #[test]
    fn test_history_database_crud() {
        let db = HistoryDatabase::open_in_memory().expect("open db");

        let record = DownloadRecord {
            id: "dl-123".to_string(),
            url: "https://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            destination_path: "/tmp/file.zip".to_string(),
            file_size: Some(1048576),
            downloaded_bytes: 1048576,
            status: "Completed".to_string(),
            elapsed_seconds: 2.5,
            average_speed: 419430.4,
            checksum: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        db.insert_download(&record).expect("insert download");
        let all = db.get_all_downloads().expect("get downloads");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "dl-123");
        assert_eq!(all[0].filename, "file.zip");
    }

    #[test]
    fn test_host_profile_tracking() {
        let db = HistoryDatabase::open_in_memory().expect("open db");

        let mut profile = HostProfile::new("speedtest.net".to_string());
        profile.record_observation(50.0 * 1024.0 * 1024.0, Duration::from_millis(30), true);

        db.upsert_host_profile(&profile).expect("upsert profile");

        let loaded = db.get_host_profile("speedtest.net").expect("get profile");
        assert!(loaded.is_some());
        let prof = loaded.unwrap();
        assert_eq!(prof.host, "speedtest.net");
        assert!(prof.average_throughput > 1024.0);
        assert_eq!(prof.range_reliability, 1.0);
    }
}
