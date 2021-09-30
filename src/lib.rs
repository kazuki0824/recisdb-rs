pub mod access_control;
pub(crate) mod inner_decoder;
mod utils;

use pin_project_lite::pin_project;
use crate::access_control::{EcmKeyHolder, EmmChannel};
use futures::AsyncRead;
use std::cell::Cell;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::mpsc::{channel, Receiver};
use crate::inner_decoder::decoder;

use once_cell::sync::OnceCell;
static mut CHANNEL: OnceCell<EmmChannel> = OnceCell::new();
static mut KEYHOLDER: OnceCell<EcmKeyHolder> = OnceCell::new();

pin_project! {
    pub struct StreamDecoder<'a> {
        #[pin]
        pub src: &'a mut (dyn AsyncRead + Unpin),
        inner: NonNull<decoder>,
        buf: [u8; 8000],
    }
    impl PinnedDrop for StreamDecoder<'_> {
        fn drop(this: Pin<&mut Self>) {
            unsafe { this.project().inner.as_mut() }.clean_up();
        }
    }
}

pub fn receive_emm() -> Option<&'static Receiver<EmmBody>> {
    unsafe {
        if let Some((_, rx)) = CHANNEL.get() {
            Some(rx)
        } else {
            None
        }
    }
}

impl<'a> StreamDecoder<'a> {
    pub fn new(src: &'a mut (dyn AsyncRead + Unpin), key: Option<WorkingKey>) -> Self {
        if let Some(pair) = key
        {
            unsafe {
                KEYHOLDER.get_or_init(|| EcmKeyHolder {
                    key_pair: Cell::from(pair),
                });
                CHANNEL.get_or_init(|| channel())
            };
            Self {
                src,
                inner: decoder::new(true).unwrap(),
                buf: [0; 8000],
            }
        }
        else {
            Self {
                src,
                inner: decoder::new(false).unwrap(),
                buf: [0; 8000],
            }
        }
    }
}



use futures::task::{Context, Poll};
use crate::access_control::types::{WorkingKey, EmmBody};

impl AsyncRead for StreamDecoder<'_> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        match this.src.poll_read(cx, &mut this.buf[0..]) {
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(n)) if n <= 0 => Poll::Ready(Ok(0)),
            Poll::Ready(Ok(n)) => unsafe {
                let recv = &mut this.buf[0..n];
                let dec = this.inner.as_mut();

                if let Some(decoded) = dec.push(recv) {
                    let contents =
                        std::ptr::slice_from_raw_parts(decoded.data, decoded.size as usize);
                    buf[0..decoded.size as usize].copy_from_slice(&*contents);
                    Poll::Ready(Ok(decoded.size as usize))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            _ => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
