use crate::state::ResumeState;
use aurora_core::error::Result;
use aurora_core::types::{Segment, ServerCapabilities};
use tracing::info;

#[derive(Debug, PartialEq, Eq)]
pub enum ResumeValidationResult {
    /// Safe to resume partial download.
    Valid {
        segments: Vec<Segment>,
        restored_bytes: u64,
    },
    /// Remote resource changed or range is unsupported; partial data must be invalidated.
    Invalid { reason: String },
}

/// Evaluates whether an existing recovery state can be safely resumed against current server capabilities.
pub fn validate_resume(
    state: &ResumeState,
    remote_caps: &ServerCapabilities,
) -> Result<ResumeValidationResult> {
    // Check 1: Range support is required for multi-segment resumption
    if !remote_caps.accepts_ranges {
        return Ok(ResumeValidationResult::Invalid {
            reason: "Server no longer supports HTTP Range requests".to_string(),
        });
    }

    // Check 2: Total file size must match if known
    if let (Some(saved_size), Some(remote_size)) = (state.file_size, remote_caps.content_length) {
        if saved_size != remote_size {
            return Ok(ResumeValidationResult::Invalid {
                reason: format!(
                    "File size changed on server (was {} bytes, now {} bytes)",
                    saved_size, remote_size
                ),
            });
        }
    }

    // Check 3: ETag validator match
    if let (Some(saved_etag), Some(remote_etag)) = (&state.etag, &remote_caps.etag) {
        if !is_etag_matching(saved_etag, remote_etag) {
            return Ok(ResumeValidationResult::Invalid {
                reason: format!(
                    "ETag validator mismatch (was {}, now {})",
                    saved_etag, remote_etag
                ),
            });
        }
    }

    // Check 4: Last-Modified validator match if ETags were not present
    if state.etag.is_none() || remote_caps.etag.is_none() {
        if let (Some(saved_lm), Some(remote_lm)) = (&state.last_modified, &remote_caps.last_modified)
        {
            if saved_lm != remote_lm {
                return Ok(ResumeValidationResult::Invalid {
                    reason: format!(
                        "Last-Modified timestamp mismatch (was {}, now {})",
                        saved_lm, remote_lm
                    ),
                });
            }
        }
    }

    // Check 5: Local destination file must exist
    if !state.destination_path.exists() {
        return Ok(ResumeValidationResult::Invalid {
            reason: format!(
                "Local file {:?} does not exist",
                state.destination_path
            ),
        });
    }

    // Convert checkpoints back to active segments
    let segments: Vec<Segment> = state.segments.iter().map(|c| c.to_segment()).collect();
    let restored_bytes = segments.iter().map(|s| s.downloaded).sum();

    info!(
        "Resume validation passed: restored {} bytes across {} segments",
        restored_bytes,
        segments.len()
    );

    Ok(ResumeValidationResult::Valid {
        segments,
        restored_bytes,
    })
}

/// Robust ETag comparison ignoring weak prefixes (W/).
fn is_etag_matching(etag_a: &str, etag_b: &str) -> bool {
    let clean_a = etag_a.trim_start_matches("W/").trim_matches('"');
    let clean_b = etag_b.trim_start_matches("W/").trim_matches('"');
    clean_a == clean_b
}
