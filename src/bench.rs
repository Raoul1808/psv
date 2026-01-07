use core::panic;
use std::{
    fs::{self, File},
    io::{Write, stdout},
    process::Command,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use inquire::{Select, prompt_u32, prompt_usize};
use rand::{rng, seq::SliceRandom};
use threadpool::ThreadPool;

use crate::{gui::SortingStrategy, sim::PushSwapSim};

pub fn benchmark() {
    let numbers = prompt_u32("Amount of numbers to sort:").expect("failed to get number");
    let tests =
        prompt_usize("Amount of tests to execute for benchmark:").expect("failed to get number");
    let strategy = Select::new("Sorting strategy:", SortingStrategy::ALL.to_vec())
        .prompt()
        .expect("failed to get sorting strategy");

    let exec_path = if let Ok(path) = fs::canonicalize("push_swap") {
        println!("Found push_swap executable in current directory");
        path
    } else {
        println!("Select path to push_swap executable");
        rfd::FileDialog::new()
            .set_title("Select push_swap executable path")
            .pick_file()
            .expect("no file selected")
    };

    let results = Arc::new(Mutex::new(vec![None; tests]));
    let error_log = File::create("error.log").expect("cannor create error.log file");
    let error_log = Arc::new(Mutex::new(error_log));
    let pool = ThreadPool::new(4);
    for test_num in 0..tests {
        let results = results.clone();
        let exec_path = exec_path.clone();
        let error_log = error_log.clone();
        pool.execute(move || {
            let mut sim = PushSwapSim::default();
            let mut numbers: Vec<_> = (0..numbers).collect();
            numbers.shuffle(&mut rng());
            let args: Vec<_> = numbers.iter().map(u32::to_string).collect();
            let mut cmd = Command::new(exec_path.clone());
            if strategy != SortingStrategy::None {
                cmd.arg(strategy.to_arg());
            }
            let instructions = cmd
                .args(args)
                .output()
                .expect("push_swap command failed to run");
            let instructions = String::from_utf8(instructions.stdout)
                .expect("push_swap output is not valid utf-8 text");
            sim.load_normalized(numbers.clone(), &instructions)
                .expect("invalid instructions");
            let mut extern_program_counter = 0;
            while sim.step() {
                extern_program_counter += 1;
            }
            sim.make_contiguous();
            if !sim.stack_a().is_sorted() || !sim.stack_b().is_empty() {
                let mut error_log = error_log.lock().expect("gimme");
                let _ = writeln!(error_log, "Test {} failed.", test_num);
                let _ = writeln!(error_log, "Numbers: {:?}", numbers);
                let _ = writeln!(error_log, "Instructions: {}", instructions);
                let _ = writeln!(
                    error_log,
                    "Final stack state: {:?} {:?}",
                    sim.stack_a(),
                    sim.stack_b()
                );
                let _ = writeln!(error_log, "====================================");
                drop(error_log);
                panic!(
                    "Test {}: Stack A is not sorted! Error details logged in error.log",
                    test_num
                );
            }
            let mut results = results.lock().expect("panic chain!");
            results[test_num] = Some(extern_program_counter);
        });
    }
    println!("Tests running.");
    let digit_width = (tests.checked_ilog10().unwrap_or(0) + 1) as usize;
    loop {
        let queued = pool.queued_count();
        print!("\rTests left: {:<width$}", queued, width = digit_width);
        stdout().flush().expect("failed to flush stdout");
        if queued == 0 {
            break;
        }
        sleep(Duration::from_secs(1));
    }
    println!();
    pool.join();
    let panics = pool.panic_count();
    if panics > 0 {
        println!(
            "{} thread(s) panicked! Check the error log to see which tests resulted in errors",
            panics
        );
    } else {
        println!("Testing done with no errors!");
    }
    let results = results.lock().expect("panic chain!");
    let results: Vec<_> = results.iter().filter_map(|r| *r).collect();
    let min = results.iter().copied().min().unwrap_or(0);
    let max = results.iter().copied().max().unwrap_or(0);
    let avg = if results.is_empty() {
        0
    } else {
        results.iter().sum::<usize>() / results.len()
    };
    println!(
        "Tests Passed: {}, Min: {}, Average: {}, Max: {}",
        results.len(),
        min,
        avg,
        max
    );
    println!(
        "Note: these values may change and can be more or less accurate depending on how many tests you ran."
    );
}
