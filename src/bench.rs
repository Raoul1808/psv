use core::panic;
use std::{
    io::{stdin, stdout, Write},
    process::Command,
    str::FromStr,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use rand::{seq::SliceRandom, thread_rng};
use threadpool::ThreadPool;

use crate::sim::PushSwapSim;

fn prompt<T>(prompt: &str) -> Result<T, T::Err>
where
    T: FromStr,
{
    println!("{}", prompt);
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("failed to read stdin");
    buf.trim_end().parse()
}

pub fn benchmark() {
    let numbers: u32 =
        prompt("Enter amount of sorting numbers to use").expect("invalid number given");
    let tests: usize = prompt("Enter amount of tests to execute").expect("invalid number given");

    println!("Select path to push_swap executable");
    let exec_path = rfd::FileDialog::new()
        .set_title("Select push_swap executable path")
        .pick_file()
        .expect("no file selected");

    let results = Arc::new(Mutex::new(vec![0; tests]));
    let pool = ThreadPool::new(4);
    for test_num in 0..tests {
        let results = results.clone();
        let exec_path = exec_path.clone();
        pool.execute(move || {
            let mut sim = PushSwapSim::default();
            let mut numbers: Vec<_> = (0..numbers).collect();
            numbers.shuffle(&mut thread_rng());
            let args: Vec<_> = numbers.iter().map(u32::to_string).collect();
            let instructions = Command::new(exec_path.clone())
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
            if !sim.stack_a().is_sorted() {
                eprintln!("Stack A is not sorted!");
                eprintln!("Starting numbers: {:?}", numbers);
                eprintln!("Instructions received: {:?}", instructions);
                panic!("shit happened!!!");
            }
            let mut results = results.lock().expect("panic chain!");
            results[test_num] = extern_program_counter;
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
        println!("{} thread(s) panicked!", panics);
    } else {
        println!("Testing done with no errors!");
    }
    let results = results.lock().expect("panic chain!");
    let min = results
        .iter()
        .min()
        .expect("there SHOULD be a minimum value in here!");
    let max = results
        .iter()
        .max()
        .expect("there SHOULD be a maximum value in here!");
    let avg = results.iter().sum::<u32>() / results.len() as u32;
    println!("Min: {}, Average: {}, Max: {}", min, avg, max);
    println!("Note: these values may change and can be more or less accurate depending on how many tests you ran.");
}
