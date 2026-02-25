# How to use

## The GUI

First, either [download (LINUX ONLY)](https://github.com/Raoul1808/psv/releases/latest/download/psv) or
[build](#building) the program.

Mark the program as executable (`chmod +x psv`), then run it (in a terminal or
by double-clicking it, it doesn't matter).

> [!NOTE]
>
> Running psv will generate a config file in the working directory. Make sure to .gitignore it!

psv's UI is split across separate floating windows:
- **The Master window**: allows you to toggle the following 3 floating windows
- **Visual Options**: purely cosmetic, you can change the visuals of psv, such as the background color or the color of sorted numbers.
- **Loading Options**: this is where you pick a number sequence and where to source sorting instructions from.
- **Playback Controls**: controls the playback of instructions. You can pause, skip and change the playback speed.

### Visual Options

You may freely change:
- The opacity of floating windows
- The scale of the user interface
- The background color
- The color of sorted numbers
- Which color profile to use

This window also comes with a color previewer, since the actual colors don't get updated unless the simulation is currently running.

### Loading Options

In this menu, you can select a number preset and a source for push\_swap instructions.

The following number presets are available:
- **Ordered**: generates an ordered list of numbers from `0` to `n-1`.
- **Reverse Ordered**: generates a reverse-ordered list of numbers from `n-1` to `0`.
- **Random Normalized**: generates an ordered list of numbers from `0` to `n-1`, then shuffle it.
- **Random from Custom Range**: generates a list of `n` random numbers within the specified range, with no repeats.
- **User Input**: allows you to input a specific sequence of numbers. Numbers must be separated by spaces. **psv does not check whether you input valid numbers or not.**
- **Preset**: a collection of fun presets.

For the Random Normalized and Random from Custom Range options, you can specify whether you want to generate
the sequence of numbers with a certain target disorder. To achieve the target disorder,
numbers are swapped randomly until the target disorder condition is met. The following settings are available:
- **Shuffle before matching disorder**: if unchecked, the sequence of numbers will be sorted before swapping numbers. Otherwise, shuffle the numbers, then start swapping.
- **Minimum amount of swaps**: how many swaps to perform before trying to match the target disorder. Should be used if the previous setting is unchecked.
- **Target disorder**: target disorder to match.

> [!WARNING]
>
> This feature is experimental and may cause sequences with unreasonable disorder settings to never generate!
> If you notice a sequence takes too long to generate, you can press the Kill button to stop it early.

Finally, you can choose one of the following 3 sources for push\_swap instructions:
- **User Input**: you will be able to input push\_swap instructions yourself.
- **File**: pick a text file containing a list of push\_swap instructions separated by whitespace.
- **Program Output**: select your push\_swap executable. psv will execute your push\_swap with the generated/specified number sequence and read instructions from the standard output of your program.

> [!TIP]
>
> psv will automatically detect your local push\_swap executable file in the working directory.

Then, click the **Visualize** button to load the numbers and instructions.

The **Visualize** button will temporarily turn into a **Kill** button that allows you to stop
the generation of numbers or the execution of your program.

If you need to generate a new sequence of numbers, simply click the **Visualize** button again.

If you need to debug a randomly generated sequence of numbers, you may click on **Copy numbers to clipboard**.
You can paste this sequence as a list of arguments for your program, and in the **User Input** number generation mode in psv.

Once done, the background will change and the Playback Controls window should appear.

### Playback Controls

The playback controls window allows you to adjust the playback of your push\_swap instructions.

The window will display how many instructions your program generated, as well as give you options to play instructions,
change the playback speed (up to 1/4th the total amount of instructions per second), skip forwards or backwards in your list of instructions,
and even see a full list of output instructions.

You can also use the spacebar to play/pause the simulation and press the left/right arrow keys to step through.


## Benchmarking

Benchmarking allows you to test the efficiency of sorting numbers by running random tests
multiple times and calculating how many instructions were needed on average.

To enter benchmarking mode, open a terminal window and run psv with the argument
`benchmark`, `bench` or `b`.

You will then be asked to provide how many numbers should be sorted, how many
tests should be run, the sorting strategy to use, and finally the path to your push\_swap executable.
Running more tests gives more accurate results, but it also takes more time.

> [!TIP]
>
> Just like when using the GUI, psv will auto-detect your push\_swap executable if psv finds it in the current directory.

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

> [!NOTE]
>
> Instructions that do not sort numbers will not be taken into account when showing the final benchmarking results.
