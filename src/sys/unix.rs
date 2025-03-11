use std::io::Result;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};

pub use tokio::net::UnixStream;

use crate::{EitherStream, Stream};

pub(super) type LeftStream = UnixStream;
pub(super) type RightStream = UnixStream;

pub async fn pipe() -> Result<(LeftStream, RightStream)> {
    UnixStream::pair()
}

impl AsFd for Stream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        multiplex!(self.as_fd())
    }
}

impl AsRawFd for Stream {
    fn as_raw_fd(&self) -> RawFd {
        multiplex!(self.as_raw_fd())
    }
}
