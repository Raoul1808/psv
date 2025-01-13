# psv

Yet another *push\_swap* visualizer written in Rust. *psv* stands for
**p**ush\_**s**wap **v**isualizer.

## Context

*push_swap* is a 42 project where the objective is to create a program that
takes a sequence of numbers as input and outputs a series of instructions to
sort those numbers by using two stacks and minimalist operations.

This program helps visualise the output of the push\_swap program.

## How to use

### The GUI

First, either [download (LINUX ONLY)](https://github.com/Raoul1808/psv/releases/latest/download/psv) or
[build](#building) the program.

Mark the program as executable (`chmod +x psv`), then run it (in a terminal or
by double-clicking it, it doesn't matter).

psv's UI is kept as simple as possible. When you open the program, only the
Loading Options window will be visible. To test your push\_swap program, first
you need to specify how random numbers should be generated.

- **Ordered**: psv will generate an ordered list of numbers from `0` to `n-1`.
- **Random Normalized**: psv will generate an ordered list of numbers from `0` to `n-1`, then shuffle it.
- **Random from Custom Range**: psv will randomly pick `n` numbers from the specified range.
- **User Input**: you will be able to input numbers yourself (useful to debug a single test case).
- **Preset**: a collection of pretty number arrangements with some arrangements that are known to break some push\_swap programs.

Once you have selected the number generation option, you will have to provide
a source of push\_swap instructions. You have 2 options:

- **User Input**: you will be able to input push\_swap instructions yourself.
- **Program Output**: give it the path to your push\_swap executable. psv will generate the random numbers and execute your program with the list of numbers as arguments.

Then, click the **Visualize** button to load the numbers and instructions.
The Playback Controls window will appear on-screen. You will be able to:

- See how many instructions are loaded
- Play, pause, step and skip through the simulation's playback to see how numbers are sorted
- Change the playback speed if you like seeing numbers being sorted but lack the patience for it
- View a list of instructions

You can also use the spacebar to play/pause the simulation and press the left/right arrow keys to step through.

If you need to test another arrangement of numbers and/or another set of
instructions, you can just load another simulation from the Loader Options window.

You can also change the look of the simulation if you'd like. Bring up the
Visual Options window, where you can change

- The floating windows' backing transparency
- The program's background color (clear color)
- How the sorted numbers should be colored based on their range

These options are purely cosmetic and do not impact in any way the sorting of numbers.

> [!NOTE]
>
> The coloring of sorted numbers may not update in real time. **THIS IS NORMAL**,
> as the vertex data is only regenerated when the simulation is running,
> or was stepped or skipped through.

### Benchmarking

psv's most useful feature is **benchmarking**. Benchmarking allows you to test
the efficiency of sorting numbers by running random tests multiple times and
calculating how many instructions were needed on average.

To enter benchmarking mode, open a terminal window and run psv with the argument
`benchmark`, `bench` or `b`.

You will then be asked to provide how many numbers should be sorted, how many
tests should be run, and finally the path to your push\_swap executable.
Running more tests gives more accurate results, but it also takes more time.

Tests are run in parallel, the number of tests left to run will appear on screen
while waiting.

When all tests are done running, the results will appear, showing the minimum
amount, maximum amount and average amount of instructions needed to sort all the
numbers.

> [!WARNING]
>
> During the benchmarking process, a log file is created. If at any point in the
> tests a set of numbers does not end up sorted, the test will panic and all
> variables used in the process will be logged to the log file.

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

> [!WARNING]
>
> `wgpu` has a fairly high MSRV requirement, so make sure your Rust toolchain is up-to-date!

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
