use std::io::{self, ErrorKind, Read};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use log::{debug, warn};

/// Default chunk size for each read from the source device (32 KiB).
/// This matches recpt1's MAX_READ_SIZE, which has proven effective for
/// tuner device reads on both BS and CS channels.
const DEFAULT_CHUNK_SIZE: usize = 32 * 1024;

/// Default queue capacity (number of chunks the channel can hold).
/// 4096 chunks × 32 KiB = 128 MiB of buffering.
/// This provides ample headroom for absorbing temporary stalls in the
/// downstream pipeline (e.g., B-CAS card ECM processing latency),
/// preventing kernel tuner buffer overflows and resultant data drops.
const DEFAULT_QUEUE_CAPACITY: usize = 4096;

const ENV_TUNER_CHUNK_SIZE_BYTES: &str = "RECISDB_TUNER_CHUNK_SIZE_BYTES";
const ENV_TUNER_QUEUE_CAPACITY: &str = "RECISDB_TUNER_QUEUE_CAPACITY";

/// Timeout in milliseconds for poll() in the reader loop.
/// The reader thread checks the shutdown flag after each timeout, so this
/// determines the maximum latency between ThreadedReader::drop() and the
/// reader thread actually exiting. 100ms provides a good balance between
/// responsiveness and avoiding excessive poll syscalls.
const POLL_TIMEOUT_MS: i32 = 100;

/// A buffered wrapper around any `Read` source that decouples the reading
/// from the consuming thread by using a dedicated background thread.
///
/// The background thread continuously reads from the source into a bounded
/// channel (producer-consumer pattern), preventing kernel device buffer
/// overflows when the consumer (e.g., B-CAS decoder pipeline) is temporarily
/// slow or blocked.
///
/// This is the same producer-consumer architecture used by recpt1 (a mature,
/// C-based tuner recording tool), which has proven effective at avoiding
/// TS packet drops even on high-bitrate CS channels with slow physical
/// B-CAS card readers.
///
/// # Lifecycle
///
/// The reader thread exits on EOF, fatal I/O error, or shutdown request.
/// On drop, `ThreadedReader` sets a shutdown flag and joins the worker.
/// `poll()` timeout is used so the worker can observe the flag even while
/// no data is available.
pub(crate) struct ThreadedReader {
    /// Receiver end of the bounded channel from the reader thread.
    receiver: Option<Receiver<io::Result<Vec<u8>>>>,
    /// Buffer for partially-consumed data from the last received chunk.
    /// When a chunk from the channel is larger than the caller's read
    /// buffer, the remainder is stored here for subsequent read() calls.
    pending: Vec<u8>,
    /// Current read offset within `pending`.
    offset: usize,
    /// Shutdown signal for the reader thread.
    /// Set to `true` in Drop to request the reader thread to exit.
    shutdown: Arc<AtomicBool>,
    /// Handle to the reader thread, joined on Drop for deterministic cleanup.
    reader_thread: Option<JoinHandle<()>>,
}

impl ThreadedReader {
    /// Create a new `ThreadedReader` that spawns a background thread to
    /// continuously read from `source`.
    ///
    /// # Arguments
    ///
    /// * `source` - Any `Read + Send + AsRawFd + 'static` source (typically
    ///   a tuner device `File`)
    /// * `chunk_size` - Number of bytes to attempt reading per iteration.
    ///   Larger values reduce syscall overhead but use more memory per chunk.
    /// * `queue_capacity` - Maximum number of chunks the bounded channel
    ///   can hold. When the queue is full, the reader thread blocks on
    ///   `send()` until the consumer drains some data. Total memory usage
    ///   is approximately `chunk_size * queue_capacity` bytes.
    pub fn new<R: Read + Send + AsRawFd + 'static>(
        source: R,
        chunk_size: usize,
        queue_capacity: usize,
    ) -> io::Result<Self> {
        let (sender, receiver) = sync_channel(queue_capacity);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);

        let reader_thread = thread::Builder::new()
            .name("tuner-reader".to_string())
            .spawn(move || {
                Self::reader_loop(source, sender, chunk_size, shutdown_clone);
            })?;

        debug!(
            "Spawned tuner reader thread (chunk_size: {} bytes, queue_capacity: {} chunks, total buffer: {} MiB)",
            chunk_size,
            queue_capacity,
            (chunk_size * queue_capacity) / (1024 * 1024),
        );

        Ok(Self {
            receiver: Some(receiver),
            pending: Vec::new(),
            offset: 0,
            shutdown,
            reader_thread: Some(reader_thread),
        })
    }

    /// Create a new `ThreadedReader` with default parameters.
    ///
    /// Uses 32 KiB chunk size and 4096-entry queue (128 MiB total buffer).
    /// Both values can be overridden by environment variables:
    /// - RECISDB_TUNER_CHUNK_SIZE_BYTES
    /// - RECISDB_TUNER_QUEUE_CAPACITY
    pub fn with_defaults<R: Read + Send + AsRawFd + 'static>(source: R) -> io::Result<Self> {
        let chunk_size = Self::read_usize_env(ENV_TUNER_CHUNK_SIZE_BYTES, DEFAULT_CHUNK_SIZE);
        let queue_capacity = Self::read_usize_env(ENV_TUNER_QUEUE_CAPACITY, DEFAULT_QUEUE_CAPACITY);
        Self::new(source, chunk_size, queue_capacity)
    }

    fn read_usize_env(name: &str, fallback: usize) -> usize {
        match std::env::var(name) {
            Ok(raw) => match raw.parse::<usize>() {
                Ok(value) if value > 0 => value,
                _ => {
                    warn!(
                        "Invalid value for {}: {}. Falling back to {}.",
                        name, raw, fallback,
                    );
                    fallback
                }
            },
            Err(_) => fallback,
        }
    }

    /// Background reader loop that continuously reads from the source
    /// and sends data chunks through the bounded channel.
    ///
    /// Uses `poll()` timeout to periodically observe `shutdown`, and bounded
    /// channel backpressure (`send()` blocks when queue is full).
    fn reader_loop<R: Read + AsRawFd>(
        mut source: R,
        sender: SyncSender<io::Result<Vec<u8>>>,
        chunk_size: usize,
        shutdown: Arc<AtomicBool>,
    ) {
        let fd = source.as_raw_fd();

        loop {
            // Check shutdown flag before doing any work
            if shutdown.load(Ordering::Relaxed) {
                break;
            }

            // Wait for data to be available on the source fd, with timeout.
            // This allows the thread to periodically check the shutdown flag
            // even when the source device is not producing data.
            let mut pollfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let poll_result = unsafe { libc::poll(&mut pollfd, 1, POLL_TIMEOUT_MS) };

            // poll() returned an error
            if poll_result < 0 {
                let poll_error = io::Error::last_os_error();
                // Interrupted by a signal (EINTR): retry immediately
                if poll_error.kind() == ErrorKind::Interrupted {
                    continue;
                }
                let _ = sender.send(Err(poll_error));
                break;
            }

            // poll() timed out: no data available yet, loop back to
            // check the shutdown flag
            if poll_result == 0 {
                continue;
            }

            // Invalid fd indicates broken stream state.
            if pollfd.revents & libc::POLLNVAL != 0 {
                let _ = sender.send(Err(io::Error::new(
                    ErrorKind::BrokenPipe,
                    "Tuner fd became invalid while polling.",
                )));
                break;
            }

            // If readable data is present, prioritize read even when HUP/ERR is
            // also set. This preserves any final bytes before stream teardown.
            if pollfd.revents & libc::POLLIN == 0 {
                if pollfd.revents & libc::POLLERR != 0 {
                    let _ = sender.send(Err(io::Error::new(
                        ErrorKind::Other,
                        "Tuner poll reported device error (POLLERR).",
                    )));
                    break;
                }
                if pollfd.revents & libc::POLLHUP != 0 {
                    let _ = sender.send(Err(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "Tuner stream hang-up detected (POLLHUP).",
                    )));
                    break;
                }
                continue;
            }

            let mut buf = vec![0u8; chunk_size];
            match source.read(&mut buf) {
                // EOF: signal completion by sending an empty Vec, then exit
                Ok(0) => {
                    let _ = sender.send(Ok(Vec::new()));
                    break;
                }
                Ok(bytes_read) => {
                    buf.truncate(bytes_read);
                    // If send fails, the receiver has been dropped
                    // (consumer is done), so we exit the loop
                    if sender.send(Ok(buf)).is_err() {
                        break;
                    }
                }
                // Interrupted by a signal (EINTR): retry immediately.
                // This is a transient condition that can occur when signals
                // (e.g., SIGALRM, SIGCHLD) are delivered during a blocking
                // read syscall. Treating it as fatal would prematurely
                // terminate tuner ingestion during long-running recordings.
                Err(ref io_error) if io_error.kind() == ErrorKind::Interrupted => {
                    warn!("Tuner read interrupted by signal (EINTR), retrying...");
                    continue;
                }
                // Fatal I/O error: forward to consumer and exit
                Err(io_error) => {
                    let _ = sender.send(Err(io_error));
                    break;
                }
            }
        }
        debug!("Tuner reader thread exiting.");
    }
}

impl Drop for ThreadedReader {
    fn drop(&mut self) {
        // Signal the reader thread to exit.
        self.shutdown.store(true, Ordering::Relaxed);

        // Drop the receiver first so that a blocked sender.send() in the
        // reader thread immediately wakes with a disconnected-channel error.
        // Without this, if the queue is full and reader thread is blocked on
        // send(), it cannot observe the shutdown flag and join() may hang.
        let _ = self.receiver.take();

        // Join the reader thread to ensure deterministic cleanup.
        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Read for ThreadedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Per the Read trait contract, a zero-length buffer must return
        // Ok(0) immediately without blocking. Without this guard, an empty
        // buf with no pending data would fall through to recv() and block.
        if buf.is_empty() {
            return Ok(0);
        }

        // First, drain any remaining data from the previously received chunk.
        // This avoids calling recv() when we already have data available,
        // which would block unnecessarily.
        if self.offset < self.pending.len() {
            let available = self.pending.len() - self.offset;
            let copy_size = available.min(buf.len());
            buf[..copy_size].copy_from_slice(&self.pending[self.offset..self.offset + copy_size]);
            self.offset += copy_size;
            // If we've consumed all pending data, clear the buffer
            if self.offset >= self.pending.len() {
                self.pending.clear();
                self.offset = 0;
            }
            return Ok(copy_size);
        }

        // No pending data; block until the reader thread sends the next chunk
        let receiver = match self.receiver.as_ref() {
            Some(receiver) => receiver,
            None => return Ok(0),
        };
        match receiver.recv() {
            // EOF signaled by the reader thread (empty Vec)
            Ok(Ok(data)) if data.is_empty() => Ok(0),
            Ok(Ok(data)) => {
                let copy_size = data.len().min(buf.len());
                buf[..copy_size].copy_from_slice(&data[..copy_size]);
                // If the received chunk is larger than buf, save the
                // remainder for subsequent read() calls
                if data.len() > copy_size {
                    self.pending = data;
                    self.offset = copy_size;
                }
                Ok(copy_size)
            }
            // I/O error forwarded from the reader thread
            Ok(Err(io_error)) => Err(io_error),
            // Sender dropped unexpectedly; treat as EOF
            Err(_) => Ok(0),
        }
    }
}
