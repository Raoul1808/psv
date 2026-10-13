use std::{
    io::{self, ErrorKind, Read},
    os::fd::AsRawFd,
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc::{Receiver, TryRecvError, channel},
    thread::{self, JoinHandle},
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
}

#[derive(Debug)]
pub struct ExecResult {
    pub duration: Duration,
    pub numbers: Vec<i32>,
    pub instructions: Vec<PushSwapInstruction>,
}

fn run_push_swap(
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

pub struct PushSwapExec {
    cancellation_token: CancellationToken,
    handle: JoinHandle<()>,
    receiver: Receiver<Result<ExecResult, ExecError>>,
    result: Option<Result<ExecResult, ExecError>>,
}

impl PushSwapExec {
    pub fn new(path: PathBuf, numbers: Vec<i32>) -> Self {
        let cancellation_token = CancellationToken::new();
        let token_clone = cancellation_token.clone();
        let (sender, receiver) = channel();
        let handle = thread::spawn(move || {
            let res = run_push_swap(path, numbers, token_clone);
            sender
                .send(res)
                .expect("there is nothing we can do about this");
        });
        Self {
            cancellation_token,
            handle,
            receiver,
            result: None,
        }
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn poll_process(&mut self) {
        match self.receiver.try_recv() {
            Ok(r) => {
                self.result = Some(r);
            }
            Err(TryRecvError::Disconnected) => {
                self.result = Some(Err(ExecError::Disconnected));
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    pub fn is_finished(&self) -> bool {
        self.result.is_some()
    }

    pub fn finish(mut self) -> Result<ExecResult, ExecError> {
        self.handle
            .join()
            .expect("something very wrong happened with joining the thread!!!");
        drop(self.receiver);
        drop(self.cancellation_token);
        self.result.take().unwrap()
    }
}
