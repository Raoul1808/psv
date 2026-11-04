use crate::help::HelpSection;

pub struct DetailsViewSection;

impl HelpSection for DetailsViewSection {
    fn section_name(&self) -> &str {
        "Details view"
    }

    fn section_ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("In the");
            let _ = ui.button("View");
            ui.label("menu, you can toggle on or off a side Details menu.");
        });
        ui.add_space(10.);
        ui.label(
            "This menu displays additional info about the current set of instructions, such as:",
        );
        ui.label("- The amount of instructions");
        ui.label("- The amount of numbers in each stack");
        ui.label("- Whether the numbers are currently sorted (this means: if all numbers are in Stack A, AND they are in the correct order. This should appear as `true` when running the last instruction)");
        ui.add_space(10.);
        ui.label("You will also find a table with all the instructions emmitted by your push_swap program. You can scroll through instructions and click on one to jump to before its execution.");
        ui.label("If the Auto-scroll option is checked, the table will scroll to the current instruction during playback.");
        ui.add_space(5.);
        ui.label("Note: if the speed is set too fast, the table will still scroll, but the focused instruction will not appear.");
    }
}
