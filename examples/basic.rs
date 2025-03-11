use std::io::Result;

use async_os_pipe::pipe;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[tokio::main]
async fn main() -> Result<()> {
    let (mut tx, mut rx) = pipe().await?;

    // write on one end
    tx.write_all(b"hello world!\n").await?;

    // read from the other
    let mut buf = vec![0; 13];
    rx.read_exact(&mut buf).await?;

    assert_eq!(buf, b"hello world!\n");

    Ok(())
}
