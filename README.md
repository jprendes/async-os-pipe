# async-os-pipe

Cross platform implementation of a bidirectional async pipe.

The `pipe` function return a pair of connected `Stream`s.
Both `Stream`s implementing `AsyncRead` and `AsyncWrite`, and are backed by an OS-level file descriptor.
In windows they are named pipes, while in unix they are anonymous unix sockets.

```rust
use std::io::Result;
use async_os_pipe::pipe;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _,};

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
```

This library can be used with [`stdio-utils`](https://crates.io/crates/stdio-utils) to redirect `stdout` in interesting ways

```rust
use std::fs::File;
use std::io::{Result, Write as _};

use async_os_pipe::pipe;
use stdio_utils::{AsFdExt as _, StdioOverride as _};
use tokio::io::{AsyncBufReadExt as _, BufReader};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = pipe().await?;

    // redirect stdout to the pipe
    let guard = tx.override_stdout()?;
    drop(tx);

    // get a copy of the original stdout
    let mut original: File = guard.duplicate_file()?.into();

    let echo_task = tokio::spawn(async move {
        // read lines from the pipe and print them on the original stdout
        let mut lines = BufReader::new(rx).lines();
        while let Some(line) = lines.next_line().await? {
            writeln!(original, "test> {line}")?;
            original.flush()?;
        }
        Result::Ok(())
    });

    // print a few things
    println!("testing stdout redirection");
    println!("each line will be prepended with \"test>\"");
    println!("bye!");

    // stop redirection
    drop(guard);

    echo_task.await??;

    Ok(())
}
```

This should print
```
test> testing stdout redirection
test> each line will be prepended with "test>"
test> bye!
```
