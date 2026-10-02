use aurora_core::types::{ByteRange, DownloadId, Segment, SegmentId, SegmentState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// Compact checkpoint representation of a single segment for disk serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SegmentCheckpoint {
    pub id: SegmentId,
    pub start: u64,
    pub end: u64,
    pub downloaded: u64,
    pub is_completed: bool,
}

impl From<&Segment> for SegmentCheckpoint {
    fn from(s: &Segment) -> Self {
        Self {
            id: s.id,
            start: s.range.start,
            end: s.range.end,
            downloaded: s.downloaded,
            is_completed: s.is_completed(),
        }
    }
}

impl SegmentCheckpoint {
    pub fn to_segment(&self) -> Segment {
        let range = ByteRange::new(self.start, self.end);
        let mut segment = Segment::new(self.id, range);
        segment.downloaded = self.downloaded;
        segment.state = if self.is_completed || self.downloaded >= range.len() {
            SegmentState::Completed
        } else {
            SegmentState::Pending
        };
        segment
    }
}

/// Complete, crash-resilient metadata required to resume a download.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResumeState {
    pub download_id: DownloadId,
    pub original_url: Url,
    pub effective_url: Url,
    pub destination_path: PathBuf,
    pub file_size: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub segments: Vec<SegmentCheckpoint>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub checksum: Option<String>,
}

impl ResumeState {
    pub fn new(
        download_id: DownloadId,
        original_url: Url,
        effective_url: Url,
        destination_path: PathBuf,
        file_size: Option<u64>,
        etag: Option<String>,
        last_modified: Option<String>,
        segments: &[Segment],
    ) -> Self {
        let now = Utc::now();
        Self {
            download_id,
            original_url,
            effective_url,
            destination_path,
            file_size,
            etag,
            last_modified,
            segments: segments.iter().map(SegmentCheckpoint::from).collect(),
            created_at: now,
            updated_at: now,
            checksum: None,
        }
    }

    pub fn total_downloaded_bytes(&self) -> u64 {
        self.segments.iter().map(|s| s.downloaded).sum()
    }
}
