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
