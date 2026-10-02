use crate::backpressure::BackpressureController;
use crate::platform;
use aurora_core::error::{Result, StorageError};
use aurora_core::traits::StorageEngine;
use bytes::Bytes;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, trace};

enum WriteOp {
    Chunk {
        offset: u64,
        data: Bytes,
        ack: Option<oneshot::Sender<()>>,
    },
    Flush {
        ack: oneshot::Sender<()>,
    },
    Shutdown {
        ack: oneshot::Sender<()>,
    },
}

/// Asynchronous, high-throughput positioned file writer with bounded memory queues.
pub struct PositionedWriter {
    path: PathBuf,
    sender: mpsc::Sender<WriteOp>,
    backpressure: BackpressureController,
    bytes_written: Arc<AtomicU64>,
    worker_handle: Option<tokio::task::JoinHandle<Result<()>>>,
}

impl PositionedWriter {
    /// Opens or creates destination file and initializes the positioned writer.
    pub async fn create(path: PathBuf, max_queue_bytes: usize) -> Result<Self> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(StorageError::Io)?;
        }

        // Open file with read/write mode, create if missing
        let std_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(StorageError::Io)?;

        let backpressure = BackpressureController::new(max_queue_bytes);
        let bp_clone = backpressure.clone();
        let bytes_written = Arc::new(AtomicU64::new(0));
        let written_counter = bytes_written.clone();

        let (tx, mut rx) = mpsc::channel::<WriteOp>(512);

        let file_path_clone = path.clone();
        let worker_handle = tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std_file;
            trace!("PositionedWriter background thread started for {:?}", file_path_clone);

            while let Some(op) = rx.blocking_recv() {
                match op {
                    WriteOp::Chunk { offset, data, ack } => {
                        let len = data.len();
                        if let Err(e) = platform::write_all_at(&file, offset, &data) {
                            error!("Write failed at offset {}: {:?}", offset, e);
                            bp_clone.record_dequeue(len);
                            return Err(e);
                        }
                        written_counter.fetch_add(len as u64, Ordering::Relaxed);
                        bp_clone.record_dequeue(len);

                        if let Some(ack_tx) = ack {
                            let _ = ack_tx.send(());
                        }
                    }
                    WriteOp::Flush { ack } => {
                        if let Err(e) = file.sync_data().map_err(StorageError::Io) {
                            error!("File sync failed: {:?}", e);
                            let _ = ack.send(());
                            return Err(e.into());
                        }
                        let _ = ack.send(());
                    }
                    WriteOp::Shutdown { ack } => {
                        let _ = file.sync_all();
                        let _ = ack.send(());
                        break;
                    }
                }
            }
            Ok(())
        });

        Ok(Self {
            path,
            sender: tx,
            backpressure,
            bytes_written,
            worker_handle: Some(worker_handle),
        })
    }

    /// Preallocates file size using platform-optimized operations.
    pub async fn preallocate_size(&self, size: u64) -> Result<()> {
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .map_err(StorageError::Io)?;
            info!("Preallocating {} bytes for file {:?}", size, path);
            platform::preallocate_file(&file, size)
        })
        .await
        .map_err(|e| StorageError::PreallocationFailed {
            size,
            reason: e.to_string(),
        })?
    }

    /// Submits a positioned chunk write.
    pub async fn submit_chunk(&self, offset: u64, data: Bytes) -> Result<()> {
        let len = data.len();
        self.backpressure.record_enqueue(len);

        // Check if backpressure threshold exceeded
        self.backpressure.wait_if_pressured().await;

        self.sender
            .send(WriteOp::Chunk {
                offset,
                data,
                ack: None,
            })
            .await
            .map_err(|_| StorageError::ChannelClosed)?;

        Ok(())
    }

    pub fn backpressure_controller(&self) -> &BackpressureController {
        &self.backpressure
    }

    pub fn total_bytes_written(&self) -> u64 {
        self.bytes_written.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl StorageEngine for PositionedWriter {
    async fn preallocate(&mut self, size: u64) -> Result<()> {
        self.preallocate_size(size).await
    }

    async fn write_chunk(&mut self, offset: u64, data: Bytes) -> Result<()> {
        self.submit_chunk(offset, data).await
    }

    async fn flush(&mut self) -> Result<()> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.sender
            .send(WriteOp::Flush { ack: ack_tx })
            .await
            .map_err(|_| StorageError::ChannelClosed)?;
        let _ = ack_rx.await;
        Ok(())
    }

    async fn finalize(&mut self) -> Result<()> {
        let (ack_tx, ack_rx) = oneshot::channel();
        let _ = self.sender.send(WriteOp::Shutdown { ack: ack_tx }).await;
        let _ = ack_rx.await;

        if let Some(handle) = self.worker_handle.take() {
            let res = handle.await.map_err(|e| StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;
            res?;
        }
        Ok(())
    }
}
