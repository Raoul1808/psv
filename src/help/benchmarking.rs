use crate::help::HelpSection;

pub struct BenchmarkingSection;

impl HelpSection for BenchmarkingSection {
    fn section_name(&self) -> &str {
        "Benchmarking"
    }

    fn section_ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("psv brings a benchmarking tool to test the performance of your push_swap program, in amount of instructions.");
        ui.add_space(15.);
        ui.label("First, set how many numbers should be sorted repeatedly. Higher numbers will yield more instructions, but it will also increase the execution time of each individual push_swap process.");
        ui.add_space(5.);
        ui.label("Then, set how many times this many numbers should be generated and sorted (AKA sorting passes). More sorting passes will yield more accurate performance results, but it will also increase the benchmarking time, although you can easily stop a benchmark at any time.");
        ui.add_space(5.);
        ui.label("Next, set the sorting strategy to be used. This option is similar to that of the Visualizer (refer to the Visualizer help section)");
        ui.add_space(5.);
        ui.label("Finally, you can set how many threads the benchmark should be run on. More threads means more push_swap process can be run in parallel, but it will spike the CPU.");
        ui.add_space(5.);
        ui.label("Note: if the computer is under heavy load, it's normal to see some executions fail! If you have a doubt, you can copy the numbers for the associated failed test and try again on the visualizer (make sure you select the User Input source)");
        ui.add_space(15.);
        ui.label("While benchmarking, you will be unable to switch app modes. For each process finshed, the benchmarking statistics and graph will be updated below.");
        ui.label("You can stop a benchmark at any time, in which case all currently running push_swap processes will be killed, queued processes will be skipped, and the current statistics will be kept, until a new benchmark is run.");
    }
}
