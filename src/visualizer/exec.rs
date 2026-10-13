use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, TryRecvError, channel},
    thread::{self, JoinHandle},
};

use crate::{
    cancellation::CancellationToken,
    exec::{ExecError, ExecResult, run_push_swap},
};

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
