use std::io::{IoSlice, IoSliceMut, Result};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};

use bytes::BufMut;
#[doc(no_inline)]
use sys::{pipe as sys_pipe, LeftStream, RightStream};
use tokio::io::{AsyncRead, AsyncWrite, Interest, ReadBuf, Ready};

enum EitherStream {
    Left(LeftStream),
    Right(RightStream),
}

/// Cross platform bidirectional stream that implements [`AsyncRead`] and [`AsyncWrite`].
pub struct Stream(EitherStream);

/// Crates a pair of connected cross-platform [Stream]'s.
///
/// ```rust
/// # use async_os_pipe::pipe;
/// # use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _,};
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let (mut tx, mut rx) = pipe().await.unwrap();
///
/// tx.write_all(b"hello world!\n").await.unwrap();
///
/// let mut buf = vec![0; 13];
/// rx.read_exact(&mut buf).await.unwrap();
///
/// assert_eq!(buf, b"hello world!\n");
/// # });
/// ```
pub async fn pipe() -> Result<(Stream, Stream)> {
    let (tx, rx) = sys_pipe().await?;
    Ok((
        Stream(EitherStream::Left(tx)),
        Stream(EitherStream::Right(rx)),
    ))
}

macro_rules! multiplex {
    ($o:ident.$($f:tt)+) => {
        match &$o.0 {
            EitherStream::Left(inner) => inner.$($f)+,
            EitherStream::Right(inner) => inner.$($f)+,
        }
    };
    (pinned, $o:ident.$($f:tt)+) => {
        match &mut $o.get_mut().0 {
            EitherStream::Left(inner) => pin!(inner).$($f)+,
            EitherStream::Right(inner) => pin!(inner).$($f)+,
        }
    };
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        multiplex!(pinned, self.poll_read(cx, buf))
    }
}

impl AsyncWrite for Stream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        multiplex!(pinned, self.poll_write(cx, buf))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        multiplex!(pinned, self.poll_flush(cx))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        multiplex!(pinned, self.poll_shutdown(cx))
    }

    fn is_write_vectored(&self) -> bool {
        multiplex!(self.is_write_vectored())
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        multiplex!(pinned, self.poll_write_vectored(cx, buf))
    }
}

impl Stream {
    /// Reads or writes from the stream using a user-provided IO operation.
    ///
    /// See [`UnixStream::async_io`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.async_io).
    pub async fn async_io<R>(&self, interest: Interest, f: impl FnMut() -> Result<R>) -> Result<R> {
        multiplex!(self.async_io(interest, f).await)
    }

    /// Waits for the stream to become readable.
    ///
    /// See [`UnixStream::readable`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.readable).
    pub async fn readable(&self) -> Result<()> {
        multiplex!(self.readable().await)
    }

    /// Waits for any of the requested ready states.
    ///
    /// See [`UnixStream::ready`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.ready).
    pub async fn ready(&self, interest: Interest) -> Result<Ready> {
        multiplex!(self.ready(interest).await)
    }

    /// Polls for read readiness.
    ///
    /// See [`UnixStream::poll_read_ready`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.poll_read_ready).
    pub fn poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        multiplex!(self.poll_read_ready(cx))
    }

    /// Tries to read or write from the socket using a user-provided IO operation.
    ///
    /// See [`UnixStream::try_io`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.try_io).
    pub fn try_io<R>(&self, interest: Interest, f: impl FnOnce() -> Result<R>) -> Result<R> {
        multiplex!(self.try_io(interest, f))
    }

    /// Try to read data from the stream into the provided buffer, returning how many bytes were read.
    ///
    /// See [`UnixStream::try_read`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.try_read).
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize> {
        multiplex!(self.try_read(buf))
    }

    /// Tries to read data from the stream into the provided buffer, advancing the buffer’s internal cursor, returning how many bytes were read.
    ///
    /// See [`UnixStream::try_read_buf`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.try_read_buf).
    pub fn try_read_buf<B: BufMut>(&self, buf: &mut B) -> Result<usize> {
        multiplex!(self.try_read_buf(buf))
    }

    /// Tries to read data from the stream into the provided buffers, returning how many bytes were read.
    ///
    /// See [`UnixStream::try_read_vectored`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.try_read_vectored).
    pub fn try_read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
        multiplex!(self.try_read_vectored(bufs))
    }

    /// Waits for the stream to become writable.
    ///
    /// See [`UnixStream::writable`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.writable).
    pub async fn writable(&self) -> Result<()> {
        multiplex!(self.writable().await)
    }

    /// Polls for write readiness.
    ///
    /// See [`UnixStream::poll_write_ready`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.poll_write_ready).
    pub fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        multiplex!(self.poll_write_ready(cx))
    }

    /// Tries to write a buffer to the stream, returning how many bytes were written.
    ///
    /// See [`UnixStream::try_write`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.try_write).
    pub fn try_write(&self, buf: &[u8]) -> Result<usize> {
        multiplex!(self.try_write(buf))
    }

    /// Tries to write several buffers to the stream, returning how many bytes were written.
    ///
    /// See [`UnixStream::try_write_vectored`](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.try_write_vectored).
    pub fn try_write_vectored(&self, buf: &[IoSlice<'_>]) -> Result<usize> {
        multiplex!(self.try_write_vectored(buf))
    }
}

#[cfg_attr(unix, path = "sys/unix.rs")]
#[cfg_attr(windows, path = "sys/windows.rs")]
mod sys;
