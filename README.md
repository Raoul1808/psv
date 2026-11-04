# psv

![GitHub Tag](https://img.shields.io/github/v/tag/Raoul1808/psv)
[Download Latest](https://github.com/Raoul1808/psv/releases/latest/download/psv)

Yet another *push\_swap* visualizer written in Rust with eframe.

psv stands for **p**ush\_**s**wap **v**isualizer.

## Context

*push_swap* is a 42 project where the objective is to create a program that takes a sequence of numbers as input and ouputs a series of instructions to sort those numbers by using two stacks and a minimalistic set of operations.

This program helps visualize and benchmark a push\_swap program.

## How to use

See Help -> Help Manual in the application.

## Features

- A clean and organized GUI
- Random number generation, normalized or from a custom range
- push\_swap instruction playback
- Customizable playback speed
- Benchmarking, with real-time plotting of results
- Color customization
- A help manual!

## Building

> [!WARNING]
>
> `wgpu` has a faily high MSRV requirement, so make sure your Rust toolchain is up-to-date!

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
