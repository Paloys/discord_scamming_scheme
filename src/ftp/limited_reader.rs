use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use pin_project::pin_project;
use tokio::io::{AsyncRead, ReadBuf};
use tokio::time::{interval, Interval};

#[pin_project]
pub struct LimitedAsyncReader<R: AsyncRead> {
    #[pin]
    inner: R,
    total_bytes: usize,
    max_bytes: usize,
}

impl<R: AsyncRead> LimitedAsyncReader<R> {
    pub fn new(inner: R, max_bytes: usize) -> Self {
        Self {
            inner,
            total_bytes: 0,
            max_bytes,
        }
    }
}

impl<R: AsyncRead> AsyncRead for LimitedAsyncReader<R> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();

        if *this.total_bytes + buf.capacity() >= *this.max_bytes {
            return Poll::Ready(Ok(()));
        }

        let before = buf.filled().len();

        let remaining_bytes = *this.max_bytes - *this.total_bytes;

        let original_unfilled = buf.remaining();
        if remaining_bytes < original_unfilled {
            buf.put_slice(&vec![0; remaining_bytes][..]);
        }

        let result = this.inner.poll_read(cx, buf);

        let after = buf.filled().len();

        let bytes_read = after - before;

        *this.total_bytes += bytes_read;

        if remaining_bytes < original_unfilled {
            buf.advance(original_unfilled - remaining_bytes);
        }

        result
    }
}
