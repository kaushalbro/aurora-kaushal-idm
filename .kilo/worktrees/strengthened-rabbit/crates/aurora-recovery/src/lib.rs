pub mod journal;
pub mod state;
pub mod validator;

pub use journal::{journal_path_for_file, load_journal, remove_journal, save_journal};
pub use state::{ResumeState, SegmentCheckpoint};
pub use validator::{validate_resume, ResumeValidationResult};

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::types::{ByteRange, DownloadId, Segment, ServerCapabilities};
    use tempfile::NamedTempFile;
    use url::Url;

    #[test]
    fn test_atomic_journal_roundtrip() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let dest_path = temp_file.path().to_path_buf();

        let seg1 = Segment::new(1, ByteRange::new(0, 499));
        let mut seg2 = Segment::new(2, ByteRange::new(500, 999));
        seg2.downloaded = 250;

        let url = Url::parse("https://example.com/test.iso").unwrap();
        let state = ResumeState::new(
            DownloadId::new(),
            url.clone(),
            url.clone(),
            dest_path.clone(),
            Some(1000),
            Some("\"abcdef\"".to_string()),
            Some("Wed, 21 Oct 2025 07:28:00 GMT".to_string()),
            &[seg1, seg2],
        );

        let journal_path = save_journal(&state).expect("save journal");
        assert!(journal_path.exists());

        let loaded = load_journal(&journal_path).expect("load journal");
        assert_eq!(loaded.destination_path, dest_path);
        assert_eq!(loaded.file_size, Some(1000));
        assert_eq!(loaded.segments.len(), 2);
        assert_eq!(loaded.segments[1].downloaded, 250);

        remove_journal(&dest_path).expect("remove journal");
        assert!(!journal_path.exists());
    }

    #[test]
    fn test_resume_validation_etag_change() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let dest_path = temp_file.path().to_path_buf();

        let url = Url::parse("https://example.com/test.iso").unwrap();
        let state = ResumeState::new(
            DownloadId::new(),
            url.clone(),
            url.clone(),
            dest_path.clone(),
            Some(1000),
            Some("\"tag-v1\"".to_string()),
            None,
            &[],
        );

        let mut remote_caps = ServerCapabilities::fallback_single_stream(url);
        remote_caps.accepts_ranges = true;
        remote_caps.content_length = Some(1000);
        remote_caps.etag = Some("\"tag-v2\"".to_string()); // ETag modified on server!

        let result = validate_resume(&state, &remote_caps).expect("validate");
        match result {
            ResumeValidationResult::Invalid { reason } => {
                assert!(reason.contains("ETag validator mismatch"));
            }
            ResumeValidationResult::Valid { .. } => panic!("Expected Invalid due to changed ETag"),
        }
    }
}
