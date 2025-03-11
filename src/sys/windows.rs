use std::io::Result;
use std::os::windows::io::{AsHandle, AsRawHandle, BorrowedHandle, RawHandle};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};

use crate::{EitherStream, Stream};

pub(super) type LeftStream = NamedPipeServer;
pub(super) type RightStream = NamedPipeClient;

const ERROR_ACCESS_DENIED: i32 = 5;
const ERROR_INVALID_PARAMETER: i32 = 87;

// inspired by https://github.com/rust-lang/rust/blob/2b285cd/library/std/src/sys/pal/windows/pipe.rs#L59
pub async fn pipe() -> Result<(LeftStream, RightStream)> {
    let mut reject_remote_clients = true;

    let mut name;
    let p1 = 'pipe: {
        for i in 0.. {
            name = make_name();
            let res = ServerOptions::new()
                .first_pipe_instance(true)
                .reject_remote_clients(reject_remote_clients)
                .max_instances(1)
                .create(&name);
            match res {
                Ok(pipe) => break 'pipe pipe,
                Err(err) if i >= 10 => {
                    // do not try forever
                    return Err(err);
                }
                Err(err) if err.raw_os_error() == Some(ERROR_ACCESS_DENIED) => {
                    // this is a name conflict, try again
                }
                Err(err) if err.raw_os_error() == Some(ERROR_INVALID_PARAMETER) => {
                    // this is an old windows version that doesn't support `reject_remote_clients`
                    // try without it
                    reject_remote_clients = false;
                }
                Err(err) => return Err(err),
            }
        }
        unreachable!();
    };

    let p2 = ClientOptions::new().open(&name).unwrap();

    p1.connect().await.unwrap();

    Ok((p1, p2))
}

fn make_name() -> String {
    static N: AtomicUsize = AtomicUsize::new(0);
    let procid = std::process::id();
    let random = N.fetch_add(1, SeqCst);
    format!(r"\\.\pipe\__async-os-pipe__.{procid}.{random}")
}

impl AsHandle for Stream {
    fn as_handle(&self) -> BorrowedHandle<'_> {
        multiplex!(self.as_handle())
    }
}

impl AsRawHandle for Stream {
    fn as_raw_handle(&self) -> RawHandle {
        multiplex!(self.as_raw_handle())
    }
}
