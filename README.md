# psv

Yet another *push\_swap* visualizer written in Rust. *psv* stands for **p**ush\_**s**wap **v**isualizer.

## Context

*push_swap* is a 42 project where the objective is to create a program that
takes a sequence of numbers as input and outputs a series of instructions to
sort those numbers by using two stacks and minimalist operations.

This program helps visualise the output of the push\_swap program.

## Features

- A nice and organised GUI
- Number sequence generation
  - Random sequence
  - User input
  - Ordered numbers (why would you need this)
- Visualise based on program output or user input
- Customisable playback speed (instructions going from 1ms to 100ms)
- Benchmarking (command-line only, run `./psv benchmark`, aliases: `bench`, `b`)
- Clear color customisation
- Sorting number color customisation
  - Gradient color customisation
  - Group color customisation
- A few number arrangement presets you can test your push\_swap on for fun!

## Building

Pre-requisites:
- An up-to-date Rust toolchain, preferrably installed using [rustup](https://rustup.rs)
- A fairly recent GPU (for running)

Steps:
1. Clone this repo
2. Run `cargo build --release` or `cargo run --release` if you want to run the program
3. ???
4. Profit

> [!NOTE]
>
> Despite the usage of WASM-ready crates, I have no plans to make and deploy
> a WebAssembly version of this program.

## License

This project is licensed under the [MIT License](LICENSE).
