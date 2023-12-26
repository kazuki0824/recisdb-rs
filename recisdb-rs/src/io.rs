use std::cell::RefCell;
use std::future::Future;
use std::io;
use std::io::Write;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{ready, Context, Poll};

use futures_util::io::{AllowStdIo, BufReader};
use futures_util::{AsyncBufRead, AsyncWrite};
use log::{error, info, warn};
use pin_project_lite::pin_project;

use b25_sys::{DecoderOptions, StreamDecoder};

pin_project! {
    pub(crate) struct AsyncInOutTriple {
        #[pin]
        i: Box<(dyn AsyncBufRead + Unpin + 'static)>,
        o: AllowStdIo<Box<dyn Write>>,
        dec: RefCell<Option<BufReader<AllowStdIo<StreamDecoder>>>>,
        amt: u64,
        abandon_decoder: Option<bool>,
        abort: Arc<AtomicBool>,
        progress_tx: std::sync::mpsc::Sender<u64>,
    }
}

impl AsyncInOutTriple {
    const CAP: usize = 1600000;
    pub fn new(
        i: Box<dyn AsyncBufRead + Unpin>,
        o: Box<dyn Write>,
        config: Option<DecoderOptions>,
        continue_on_error: bool,
    ) -> (Self, std::sync::mpsc::Receiver<u64>) {
        let raw = config.and_then(|op| match StreamDecoder::new(op) {
            Ok(raw) => Some(raw),
            Err(e) if continue_on_error => {
                error!("Failed to initialize the decoder. ({})", e);
                info!("Disabling decoding and continue...");
                // As a fallback, disable decoding and continue processing
                None
            }
            Err(e) => {
                error!("Error occurred while initializing the decoder. ({})", e);
                error!(
                    "Make sure that the B-CAS card is certainly connected, or consider using -k."
                );
                std::process::exit(-1)
            }
        });

        let dec = {
            let buffered_decoder = raw
                .map(AllowStdIo::new)
                .map(|raw| BufReader::with_capacity(Self::CAP, raw));

            RefCell::new(buffered_decoder)
        };

        let o = AllowStdIo::new(o);

        let abort: Arc<AtomicBool> = Default::default();
        let weak = Arc::downgrade(&abort);
        ctrlc::set_handler(move || {
            if let Some(ptr) = weak.upgrade() {
                ptr.store(true, Ordering::Relaxed)
            }
        })
        .expect("Error setting Ctrl-C handler");

        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
        (
            Self {
                i,
                o,
                dec,
                amt: 0,
                abort,
                progress_tx,
                abandon_decoder: if continue_on_error { Some(false) } else { None },
            },
            progress_rx,
        )
    }
}

impl Future for AsyncInOutTriple {
    type Output = io::Result<u64>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let _ = this.progress_tx.send(*this.amt);

        match this.dec.get_mut() {
            Some(ref mut dec) if !this.abandon_decoder.unwrap_or(false) => {
                //    A.         B.
                // In -> Decoder -> Out
                if !this.abort.load(Ordering::Relaxed) {
                    // A(source)
                    let buffer = ready!(this.i.as_mut().poll_fill_buf(cx))?;
                    if buffer.is_empty() {
                        // go to finalization
                        this.abort.store(true, Ordering::Relaxed);
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    } else {
                        // A(sink)
                        match ready!(Pin::new(&mut *dec).poll_write(cx, buffer)) {
                            Ok(0) => return Poll::Ready(Err(io::ErrorKind::WriteZero.into())),
                            Ok(i) => {
                                *this.amt += i as u64;
                                this.i.as_mut().consume(i);
                            }
                            Err(e) if this.abandon_decoder.is_some() => {
                                // Enable bypassing a decoder
                                *this.abandon_decoder = Some(true);
                                error!("Unexpected failure in the decoder({}).", e);
                                warn!("Falling back to decoder-less mode...")
                            }
                            Err(e) => return Poll::Ready(Err(e)),
                        }
                    }

                    // B(source)
                    let buffer = ready!(Pin::new(&mut *dec).poll_fill_buf(cx))?;
                    if !buffer.is_empty() {
                        // B(sink)
                        let j = ready!(Pin::new(&mut this.o).poll_write(cx, buffer))?;
                        if j == 0 {
                            return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                        }
                        Pin::new(&mut *dec).consume(j);
                    }

                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }

                // Finalize
                for _ in 1..1000000 {
                    match this.progress_tx.send(u64::MAX) {
                        Ok(_) => {}
                        Err(_) => {
                            // Most likely due to pressing Ctrl+C
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::Interrupted,
                                "Ctrl+C pressed",
                            )));
                        }
                    }
                }
                info!("Flushing the buffer…");

                // A(sink)
                ready!(Pin::new(&mut *dec).poll_flush(cx))?;
                // B(source)
                loop {
                    match Pin::new(&mut *dec).poll_fill_buf(cx) {
                        Poll::Ready(Ok(buffer)) if buffer.is_empty() => {
                            return Poll::Ready(Ok(*this.amt));
                        }
                        Poll::Ready(Ok(buffer)) => {
                            // B(sink)
                            let j = ready!(Pin::new(&mut this.o).poll_write(cx, buffer))?;
                            if j == 0 {
                                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                            }
                            Pin::new(&mut *dec).consume(j);
                        }
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => continue,
                    }
                }
            }
            _ => {
                // pass through
                let buffer = ready!(this.i.as_mut().poll_fill_buf(cx))?;
                if buffer.is_empty() || this.abort.load(Ordering::Relaxed) {
                    ready!(Pin::new(&mut this.o).poll_flush(cx))?;
                    return Poll::Ready(Ok(*this.amt));
                }

                let i = ready!(Pin::new(&mut this.o).poll_write(cx, buffer))?;
                if i == 0 {
                    return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                }
                *this.amt += i as u64;
                this.i.as_mut().consume(i);

                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
