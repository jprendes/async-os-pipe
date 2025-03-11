use std::io::{stdout, Write};

use async_os_pipe::*;
use stdio_utils::StdioOverride as _;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[tokio::test]
async fn smoke_test() {
    let (mut tx, mut rx) = pipe().await.unwrap();
    tx.write_all(b"hello world!\n").await.unwrap();
    let mut buf = vec![0; 13];
    rx.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, b"hello world!\n");
}

#[tokio::test]
async fn pipe_stdout() {
    let mut stdout = stdout().lock();
    let (tx, mut rx) = pipe().await.unwrap();
    {
        let _guard = tx.override_stdout().unwrap();
        writeln!(stdout, "hello world!").unwrap();
    }
    let mut buf = vec![0; 13];
    rx.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, b"hello world!\n");
}
