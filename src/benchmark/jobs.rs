use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, TryRecvError, channel},
};

use rand::seq::SliceRandom;
use threadpool::ThreadPool;

use crate::{
    benchmark::{JobResult, JobResultType},
    cancellation::CancellationToken,
    common::SortingStrategy,
    exec::{ExecError, run_push_swap},
    sim::Sim,
};

pub struct JobManager {
    job_pool: threadpool::ThreadPool,
    receiver: Receiver<JobResult>,
    cancellation_token: CancellationToken,
    disconnected: bool,
    jobs_queued: usize,
    jobs_finished: usize,
}

impl JobManager {
    pub fn run(
        path: PathBuf,
        numbers: usize,
        tests: usize,
        threads: usize,
        sorting_strategy: SortingStrategy,
    ) -> Self {
        let (sender, receiver) = channel();
        let cancellation_token = CancellationToken::new();
        let pool = ThreadPool::new(threads);
        for i in 0..tests {
            let path = path.clone();
            let sender = sender.clone();
            let cancellation_token = cancellation_token.clone();
            pool.execute(move || {
                if cancellation_token.is_cancelled() {
                    return;
                }
                let mut numbers: Vec<_> = (0..numbers as i32).collect();
                numbers.shuffle(&mut rand::rng());
                let response =
                    match run_push_swap(path, numbers, sorting_strategy, cancellation_token) {
                        Ok(res) => {
                            let mut sim =
                                Sim::new(res.numbers.as_slice(), res.instructions).unwrap();
                            while sim.step() {}
                            if sim.is_sorted() {
                                JobResultType::Success {
                                    instructions: sim.instructions().len(),
                                }
                            } else {
                                JobResultType::Failure {
                                    numbers: res.numbers,
                                }
                            }
                        }
                        Err(e) => {
                            if matches!(e, ExecError::Cancelled) {
                                return;
                            }
                            JobResultType::Error(e.to_string())
                        }
                    };
                sender
                    .send(JobResult {
                        id: i,
                        ty: response,
                    })
                    .expect("kaboom!!!!!");
            });
        }
        Self {
            job_pool: pool,
            receiver,
            cancellation_token,
            disconnected: false,
            jobs_queued: tests,
            jobs_finished: 0,
        }
    }

    pub fn cancel(&self) {
        log::info!("sending cancel signal");
        self.cancellation_token.cancel();
    }

    pub fn is_done(&self) -> bool {
        self.disconnected
    }

    pub fn jobs_progress(&self) -> f32 {
        self.jobs_finished as f32 / self.jobs_queued as f32
    }

    pub fn jobs_finished(&self) -> usize {
        self.jobs_finished
    }

    pub fn jobs_queued(&self) -> usize {
        self.jobs_queued
    }

    pub fn try_get_result(&mut self) -> Option<JobResult> {
        match self.receiver.try_recv() {
            Ok(res) => {
                self.jobs_finished += 1;
                Some(res)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.disconnected = true;
                None
            }
        }
    }

    pub fn finalize(self) {
        self.job_pool.join();
    }
}
