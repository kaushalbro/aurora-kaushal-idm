pub mod backpressure;
pub mod integrity;
pub mod platform;
pub mod writer;

pub use backpressure::BackpressureController;
pub use integrity::{compute_sha256, verify_file_checksum, HashAlgorithm};
pub use writer::PositionedWriter;

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::traits::StorageEngine;
    use bytes::Bytes;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_positioned_writer_random_offsets() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let path = temp_file.path().to_path_buf();

        let mut writer = PositionedWriter::create(path.clone(), 1024 * 1024)
            .await
            .expect("create writer");

        // Preallocate 1000 bytes
        writer.preallocate(1000).await.expect("preallocate");

        // Write chunk 2 at offset 500
        let chunk2 = Bytes::from_static(b"WORLD_CHUNK_TWO");
        writer.write_chunk(500, chunk2.clone()).await.expect("write chunk 2");

        // Write chunk 1 at offset 0
        let chunk1 = Bytes::from_static(b"HELLO_CHUNK_ONE");
        writer.write_chunk(0, chunk1.clone()).await.expect("write chunk 1");

        writer.flush().await.expect("flush");
        writer.finalize().await.expect("finalize");

        // Read and verify bytes directly from file
        let content = std::fs::read(&path).expect("read file");
        assert_eq!(&content[0..chunk1.len()], b"HELLO_CHUNK_ONE");
        assert_eq!(&content[500..500 + chunk2.len()], b"WORLD_CHUNK_TWO");
        assert_eq!(content.len(), 1000);
    }

    #[test]
    fn test_integrity_hash_verification() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        std::fs::write(temp_file.path(), b"AURORA_INTEGRITY_TEST_DATA").expect("write data");

        let sha256_hash = compute_sha256(temp_file.path()).expect("compute sha256");
        assert!(!sha256_hash.is_empty());

        let is_valid = verify_file_checksum(temp_file.path(), &sha256_hash, HashAlgorithm::Sha256)
            .expect("verify hash");
        assert!(is_valid);
    }
}
