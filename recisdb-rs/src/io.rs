use std::cell::RefCell;
use std::future::Future;
use std::io;
use std::io::Write;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

use futures_util::io::{AllowStdIo, BufReader};
use futures_util::{AsyncBufRead, AsyncWrite};
use pin_project_lite::pin_project;

use b25_sys::{DecoderOptions, StreamDecoder};

pin_project! {
    pub(crate) struct AsyncInOutTriple {
        #[pin]
        i: Box<(dyn AsyncBufRead + Unpin + 'static)>,
        o: AllowStdIo<Box<dyn Write>>,
        dec: RefCell<Option<BufReader<AllowStdIo<StreamDecoder>>>>,
        amt: u64,
    }
}

impl AsyncInOutTriple {
    const CAP: usize = 16000000;
    pub fn new(
        i: Box<dyn AsyncBufRead + Unpin>,
        o: Box<dyn Write>,
        dec: Option<DecoderOptions>,
    ) -> Self {
        let dec = {
            let buffered_decoder = dec
                .map(StreamDecoder::new)
                .map(AllowStdIo::new)
                .map(|raw| BufReader::with_capacity(Self::CAP, raw));

            RefCell::new(buffered_decoder)
        };

        let o = AllowStdIo::new(o);

        Self { i, o, dec, amt: 0 }
    }
}

impl Future for AsyncInOutTriple {
    type Output = io::Result<u64>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        match this.dec.get_mut() {
            None => {
                // pass through
                loop {
                    let buffer = ready!(this.i.as_mut().poll_fill_buf(cx))?;
                    if buffer.is_empty() {
                        ready!(Pin::new(&mut this.o).poll_flush(cx))?;
                        return Poll::Ready(Ok(*this.amt));
                    }

                    let i = ready!(Pin::new(&mut this.o).poll_write(cx, buffer))?;
                    if i == 0 {
                        return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                    }
                    *this.amt += i as u64;
                    this.i.as_mut().consume(i);
                }
            }
            Some(ref mut dec) => {
                //    A.         B.
                // In -> Decoder -> Out
                loop {
                    // A(source)
                    let buffer = ready!(this.i.as_mut().poll_fill_buf(cx))?;
                    if buffer.is_empty() {
                        break;
                    }
                    // A(sink)
                    let i = ready!(Pin::new(&mut *dec).poll_write(cx, buffer))?;
                    if i == 0 {
                        return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                    }
                    *this.amt += i as u64;
                    this.i.as_mut().consume(i);
                    // B(source)
                    let buffer = ready!(Pin::new(&mut *dec).poll_fill_buf(cx))?;
                    if buffer.is_empty() {
                        continue;
                    }
                    // B(sink)
                    let j = ready!(Pin::new(&mut this.o).poll_write(cx, buffer))?;
                    if j == 0 {
                        return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                    }
                    Pin::new(&mut *dec).consume(j);
                }

                // Finalize
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
        }
    }
}
