use std::{
    io::{self, Read},
    os::{fd::AsRawFd, unix::process::ExitStatusExt},
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use thiserror::Error;

use crate::{
    cancellation::CancellationToken,
    common::SortingStrategy,
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

fn set_nonblocking_fd(fd: std::os::unix::io::RawFd) {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
}

pub fn run_push_swap(
    path: PathBuf,
    numbers: Vec<i32>,
    sorting_strategy: SortingStrategy,
    cancellation_token: CancellationToken,
) -> Result<ExecResult, ExecError> {
    let start = Instant::now();

    let args = numbers.iter().map(|i| i.to_string());
    let mut cmd = Command::new(path);
    if let Some(arg) = sorting_strategy.to_arg() {
        cmd.arg(arg);
    }
    let mut cmd = cmd.args(args).stdout(Stdio::piped()).spawn()?;
    let mut stdout = cmd.stdout.take().expect("what the hell???");
    set_nonblocking_fd(stdout.as_raw_fd());

    let mut buffer = vec![];

    const BLOCK_SIZE: usize = 65535;

    'read_loop: loop {
        loop {
            let mut buf = vec![0; BLOCK_SIZE];
            match stdout.read(&mut buf[0..BLOCK_SIZE]) {
                Ok(0) => {
                    break 'read_loop;
                }
                Ok(n) => {
                    buffer.extend_from_slice(&buf[0..n]);
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    cmd.kill()?;
                    cmd.wait()?;
                    return Err(e.into());
                }
            }
        }

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

        if cancellation_token.is_cancelled() {
            cmd.kill()?;
            cmd.wait()?;
            return Err(ExecError::Cancelled);
        }

        std::thread::sleep(Duration::from_millis(5));
    }

    let status = cmd.wait()?;
    if let Some(status) = status.code()
        && status != 0
    {
        return Err(ExecError::Code(status));
    }
    if let Some(signal) = status.signal() {
        return Err(ExecError::Signal(signal));
    }

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
