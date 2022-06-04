mod access_control;
mod bindings;

use std::cell::Cell;
use std::io::{Read, Write};
use std::pin::Pin;

pub use futures;
pub use crate::access_control::types::WorkingKey;
use crate::bindings::InnerDecoder;
use futures::task::{Context, Poll};
use futures::{ready, AsyncBufRead, AsyncRead};
use pin_project_lite::pin_project;

pin_project! {
    pub struct StreamDecoder<'a> {
        #[pin]
        pub reader: &'a mut (dyn AsyncBufRead + Unpin),
        received: Cell<usize>,
        sent: Cell<usize>,
        inner: InnerDecoder,
    }
    impl PinnedDrop for StreamDecoder<'_> {
        fn drop(this: Pin<&mut Self>) {
            eprintln!("{}B received, and {}B converted.", this.received.get(), this.sent.get());
        }
    }
}

impl<'a> StreamDecoder<'a> {
    pub fn new(
        reader: &'a mut (dyn AsyncBufRead + Unpin),
        key: Option<WorkingKey>,
        ids: Vec<i64>,
    ) -> Self {
        unsafe {
            Self {
                received: Cell::new(0),
                sent: Cell::new(0),
                reader,
                inner: InnerDecoder::new(key).unwrap(),
            }
        }
    }
}

impl AsyncRead for StreamDecoder<'_> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut this = self.project();

        //try receiving
        let recv = ready!(this.reader.as_mut().poll_fill_buf(cx))?;
        //get n
        let n = recv.len();
        //if 0, exit, or continue waiting for next
        if n == 0 {
            this.reader.as_mut().consume(0);
            return Poll::Ready(Ok(0));
            // cx.waker().wake_by_ref();
            // Poll::Pending
        } else {
            //Write to this.inner in order to decode, and read from this.inner in order to write to buf
            this.inner.write(recv).unwrap();
            this.reader.as_mut().consume(n);
            this.received.set(this.received.get() + n);

            //try reading
            let read = this.inner.read(buf);
            //if 0, exit, or continue waiting for next
            match read {
                Ok(0) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Ok(n) => {
                    this.sent.set(this.sent.get() + n);
                    Poll::Ready(Ok(n))
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        }
    }
}
