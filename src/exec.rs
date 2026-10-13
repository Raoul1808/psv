use std::{
    io::{self, ErrorKind, Read},
    os::{fd::AsRawFd, unix::process::ExitStatusExt},
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use thiserror::Error;

use crate::{
    cancellation::CancellationToken,
    sim::{PushSwapError, PushSwapInstruction},
};

// TODO: expand errors
#[derive(Debug, Error)]
pub enum ExecError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("operation cancelled")]
    Cancelled,

    #[error("channel disconnected")]
    Disconnected,

    #[error("instruction {0}: {1}")]
    PushSwap(usize, PushSwapError),

    #[error("received non UTF-8 data")]
    Utf8,

    #[error("program exit with code {0}")]
    Code(i32),

    #[error("program received signal {0}")]
    Signal(i32),
}

#[derive(Debug)]
pub struct ExecResult {
    pub duration: Duration,
    pub numbers: Vec<i32>,
    pub instructions: Vec<PushSwapInstruction>,
}

pub fn run_push_swap(
    path: PathBuf,
    numbers: Vec<i32>,
    cancellation_token: CancellationToken,
) -> Result<ExecResult, ExecError> {
    let start = Instant::now();

    let mut poll = Poll::new()?;
    let args = numbers.iter().map(|i| i.to_string());
    let mut cmd = Command::new(path)
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdout = cmd.stdout.take().expect("what the fuck???");
    let mut stdout_fd = SourceFd(&stdout.as_raw_fd());
    poll.registry()
        .register(&mut stdout_fd, Token(0), Interest::READABLE)?;

    let mut buffer = vec![];
    let mut events = Events::with_capacity(1024);

    'read_loop: loop {
        poll.poll(&mut events, Some(Duration::from_millis(50)))?;

        if let Some(status) = cmd.try_wait()? {
            if let Some(status) = status.code()
                && status != 0
            {
                return Err(ExecError::Code(status));
            }
            if let Some(signal) = status.signal() {
                return Err(ExecError::Signal(signal));
            }
            break;
        }

        for event in &events {
            if event.token() == Token(0) && event.is_readable() {
                let mut read = vec![0; 1024];
                loop {
                    match stdout.read(&mut read[0..1024]) {
                        Ok(0) => {
                            break 'read_loop;
                        }
                        Ok(n) => {
                            buffer.extend_from_slice(&read[0..n]);
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                            continue;
                        }
                        Err(e) => {
                            cmd.kill()?;
                            cmd.wait()?;
                            return Err(e.into());
                        }
                    }
                }
            }
        }

        if cancellation_token.is_cancelled() {
            cmd.kill()?;
            cmd.wait()?;
            return Err(ExecError::Cancelled);
        }
    }

    cmd.wait()?;

    let string = String::from_utf8(buffer).map_err(|_| ExecError::Utf8)?;

    let instructions = string
        .split_whitespace()
        .enumerate()
        .map(|(i, s)| {
            s.parse::<PushSwapInstruction>()
                .map_err(|e| ExecError::PushSwap(i, e))
        })
        .collect::<Result<Vec<_>, ExecError>>()?;

    let end = Instant::now();
    let duration = end - start;
    Ok(ExecResult {
        duration,
        numbers,
        instructions,
    })
}
