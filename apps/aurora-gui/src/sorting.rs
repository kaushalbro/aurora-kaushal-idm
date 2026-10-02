use crate::app::GuiDownloadItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortColumn {
    Name,
    Size,
    Status,
    Speed,
    TimeAdded,
    TimeTook,
    Eta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
    None,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::None => SortDirection::Ascending,
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::None,
        }
    }

    pub fn arrow(&self) -> &'static str {
        match self {
            SortDirection::Ascending => " ▲",
            SortDirection::Descending => " ▼",
            SortDirection::None => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CategoryFilter {
    All,
    Downloading,
    Completed,
    Paused,
    Error,
    Compressed,
    Video,
    Audio,
    Documents,
    Programs,
}

impl CategoryFilter {
    pub fn label(&self) -> &'static str {
        match self {
            CategoryFilter::All => "All Downloads",
            CategoryFilter::Downloading => "Downloading",
            CategoryFilter::Completed => "Completed",
            CategoryFilter::Paused => "Paused / Queued",
            CategoryFilter::Error => "Failed / Errors",
            CategoryFilter::Compressed => "Compressed",
            CategoryFilter::Video => "Video & Media",
            CategoryFilter::Audio => "Audio & Music",
            CategoryFilter::Documents => "Documents",
            CategoryFilter::Programs => "Programs & Apps",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CategoryFilter::All => "📥",
            CategoryFilter::Downloading => "⚡",
            CategoryFilter::Completed => "✓",
            CategoryFilter::Paused => "⏸",
            CategoryFilter::Error => "⚠",
            CategoryFilter::Compressed => "📦",
            CategoryFilter::Video => "🎬",
            CategoryFilter::Audio => "🎵",
            CategoryFilter::Documents => "📄",
            CategoryFilter::Programs => "⚙️",
        }
    }

    pub fn matches(&self, item: &GuiDownloadItem) -> bool {
        match self {
            CategoryFilter::All => true,
            CategoryFilter::Downloading => item.is_active,
            CategoryFilter::Completed => item.status == "Completed",
            CategoryFilter::Paused => item.status == "Paused",
            CategoryFilter::Error => item.status.starts_with("Error"),
            CategoryFilter::Compressed => {
                let l = item.filename.to_lowercase();
                l.ends_with(".zip") || l.ends_with(".tar") || l.ends_with(".gz") || l.ends_with(".7z") || l.ends_with(".rar") || l.ends_with(".iso") || l.ends_with(".dmg")
            }
            CategoryFilter::Video => {
                let l = item.filename.to_lowercase();
                l.ends_with(".mp4") || l.ends_with(".mkv") || l.ends_with(".avi") || l.ends_with(".mov") || l.ends_with(".webm") || l.ends_with(".flv")
            }
            CategoryFilter::Audio => {
                let l = item.filename.to_lowercase();
                l.ends_with(".mp3") || l.ends_with(".flac") || l.ends_with(".wav") || l.ends_with(".aac") || l.ends_with(".m4a") || l.ends_with(".ogg")
            }
            CategoryFilter::Documents => {
                let l = item.filename.to_lowercase();
                l.ends_with(".pdf") || l.ends_with(".doc") || l.ends_with(".docx") || l.ends_with(".txt") || l.ends_with(".epub") || l.ends_with(".csv")
            }
            CategoryFilter::Programs => {
                let l = item.filename.to_lowercase();
                l.ends_with(".exe") || l.ends_with(".deb") || l.ends_with(".rpm") || l.ends_with(".appimage") || l.ends_with(".msi") || l.ends_with(".bin")
            }
        }
    }
}

/// Filter and sort a list of download items, returning their indices into the master list
pub fn filter_and_sort_indices(
    items: &[GuiDownloadItem],
    category: CategoryFilter,
    search: &str,
    sort_col: SortColumn,
    sort_dir: SortDirection,
) -> Vec<usize> {
    let clean_search = search.trim().to_lowercase();

    // 1. Filter
    let mut indices: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            if !category.matches(item) {
                return false;
            }
            if !clean_search.is_empty() {
                let fn_match = item.filename.to_lowercase().contains(&clean_search);
                let url_match = item.url.to_lowercase().contains(&clean_search);
                let host_match = item.server_name.as_deref().unwrap_or("").to_lowercase().contains(&clean_search);
                if !fn_match && !url_match && !host_match {
                    return false;
                }
            }
            true
        })
        .map(|(idx, _)| idx)
        .collect();

    // 2. Sort
    if sort_dir != SortDirection::None {
        indices.sort_by(|&a, &b| {
            let item_a = &items[a];
            let item_b = &items[b];

            let ordering = match sort_col {
                SortColumn::Name => item_a.filename.to_lowercase().cmp(&item_b.filename.to_lowercase()),
                SortColumn::Size => {
                    let size_a = item_a.total_bytes.unwrap_or(item_a.downloaded_bytes);
                    let size_b = item_b.total_bytes.unwrap_or(item_b.downloaded_bytes);
                    size_a.cmp(&size_b)
                }
                SortColumn::Status => {
                    let pct_a = match item_a.total_bytes {
                        Some(t) if t > 0 => item_a.downloaded_bytes as f64 / t as f64,
                        _ => if item_a.status == "Completed" { 1.0 } else { 0.0 },
                    };
                    let pct_b = match item_b.total_bytes {
                        Some(t) if t > 0 => item_b.downloaded_bytes as f64 / t as f64,
                        _ => if item_b.status == "Completed" { 1.0 } else { 0.0 },
                    };
                    pct_a.partial_cmp(&pct_b).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::Speed => {
                    item_a.current_speed.partial_cmp(&item_b.current_speed).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::TimeAdded => {
                    item_a.start_instant.cmp(&item_b.start_instant)
                }
                SortColumn::TimeTook => {
                    let took_a = item_a.elapsed_duration.unwrap_or_else(|| item_a.start_instant.elapsed());
                    let took_b = item_b.elapsed_duration.unwrap_or_else(|| item_b.start_instant.elapsed());
                    took_a.cmp(&took_b)
                }
                SortColumn::Eta => {
                    let eta_a = item_a.eta.unwrap_or(std::time::Duration::from_secs(u64::MAX));
                    let eta_b = item_b.eta.unwrap_or(std::time::Duration::from_secs(u64::MAX));
                    eta_a.cmp(&eta_b)
                }
            };

            if sort_dir == SortDirection::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::types::DownloadId;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    fn make_test_item(filename: &str, size: Option<u64>, speed: f64, status: &str) -> GuiDownloadItem {
        GuiDownloadItem {
            id: DownloadId::new(),
            url: format!("https://example.com/{}", filename),
            filename: filename.to_string(),
            destination_path: format!("/downloads/{}", filename),
            total_bytes: size,
            downloaded_bytes: if status == "Completed" { size.unwrap_or(100) } else { 0 },
            current_speed: speed,
            peak_speed: speed,
            min_speed: 0.0,
            eta: None,
            status: status.to_string(),
            active_connections: 16,
            http_version: "HTTP/2".to_string(),
            etag: None,
            accepts_ranges: true,
            is_active: status == "Downloading",
            segments: Vec::new(),
            pause_signal: Arc::new(AtomicBool::new(false)),
            cancel_signal: Arc::new(AtomicBool::new(false)),
            requested_pause: false,
            requested_resume: false,
            requested_restart: false,
            requested_open_file: false,
            requested_open_folder: false,
            requested_remove: false,
            requested_delete_file: false,
            requested_reprobe: false,
            requested_inspect: false,
            start_time_str: "12:00:00".to_string(),
            start_instant: std::time::Instant::now(),
            end_time_str: None,
            elapsed_duration: None,
            server_rtt_ms: Some(25),
            server_name: Some("Cloudflare".to_string()),
            health_rating: Some("A+".to_string()),
            checksum_input: String::new(),
            checksum_result: None,
        }
    }

    #[test]
    fn test_category_matching() {
        let zip_item = make_test_item("archive.zip", Some(1000), 500.0, "Downloading");
        let mp4_item = make_test_item("movie.mp4", Some(5000), 1000.0, "Completed");
        let pdf_item = make_test_item("doc.pdf", Some(200), 0.0, "Paused");

        assert!(CategoryFilter::All.matches(&zip_item));
        assert!(CategoryFilter::Compressed.matches(&zip_item));
        assert!(!CategoryFilter::Video.matches(&zip_item));

        assert!(CategoryFilter::Video.matches(&mp4_item));
        assert!(CategoryFilter::Completed.matches(&mp4_item));

        assert!(CategoryFilter::Documents.matches(&pdf_item));
        assert!(CategoryFilter::Paused.matches(&pdf_item));
    }

    #[test]
    fn test_filter_and_sort_indices() {
        let items = vec![
            make_test_item("zebra.tar.gz", Some(5000), 10.0, "Downloading"),
            make_test_item("alpha.mp4", Some(1000), 500.0, "Downloading"),
            make_test_item("beta.iso", Some(10000), 100.0, "Completed"),
        ];

        // Sort by Name Ascending
        let indices = filter_and_sort_indices(&items, CategoryFilter::All, "", SortColumn::Name, SortDirection::Ascending);
        assert_eq!(indices, vec![1, 2, 0]); // alpha (1), beta (2), zebra (0)

        // Sort by Size Descending
        let indices_size = filter_and_sort_indices(&items, CategoryFilter::All, "", SortColumn::Size, SortDirection::Descending);
        assert_eq!(indices_size, vec![2, 0, 1]); // beta (10000), zebra (5000), alpha (1000)

        // Filter by search query
        let indices_search = filter_and_sort_indices(&items, CategoryFilter::All, "mp4", SortColumn::Name, SortDirection::Ascending);
        assert_eq!(indices_search, vec![1]);
    }
}

