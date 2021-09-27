mod access_control;
pub mod inner_decoder;
mod utils;

use crate::access_control::{EcmKeyHolder, EmmBody};
use futures::AsyncRead;
use inner_decoder::decoder;
use std::cell::Cell;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::mpsc::{channel, Receiver, Sender};

use once_cell::sync::OnceCell;
static mut CHANNEL: OnceCell<(Sender<EmmBody>, Receiver<EmmBody>)> = OnceCell::new();
static mut KEYHOLDER: OnceCell<EcmKeyHolder> = OnceCell::new();

pub struct StreamDecoder {
    pub src: Pin<Box<dyn AsyncRead + Unpin>>,
    pub emm_channel: Option<Sender<EmmBody>>,
    inner: NonNull<decoder>,
    buf: [u8; 8000],
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

impl StreamDecoder {
    pub fn new_w_emm(src: Pin<Box<dyn AsyncRead + Unpin>>, key: WorkingKey) -> Self {
        let emm_channel = unsafe {
            KEYHOLDER.get_or_init(|| EcmKeyHolder {
                key_pair: Cell::from(key),
            });
            let (tx, _) = CHANNEL.get_or_init(|| channel());
            Some(tx.clone())
        };
        Self {
            src,
            emm_channel,
            inner: decoder::new(true).unwrap(),
            buf: [0; 8000],
        }
    }
    pub fn new(src: Pin<Box<dyn AsyncRead + Unpin>>) -> Self {
        Self {
            src,
            emm_channel: None,
            inner: decoder::new(false).unwrap(),
            buf: [0; 8000],
        }
    }
}

impl Drop for StreamDecoder {
    fn drop(&mut self) {
        unsafe { self.inner.as_mut() }.clean_up();
    }
}

use crate::utils::WorkingKey;
use futures::task::{Context, Poll};

impl AsyncRead for StreamDecoder {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        match this.src.as_mut().poll_read(cx, &mut this.buf[0..]) {
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
