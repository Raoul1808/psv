use eframe::egui;
use egui_plot::{BoxElem, BoxPlot, BoxSpread, Legend, Span, VLine};

use crate::help::HelpSection;

pub struct BenchmarkingStatsSection;

impl BenchmarkingStatsSection {
    pub fn draw_plot(&self, ui: &mut egui::Ui) {
        let plot = BoxPlot::new(
            "Instructions",
            vec![BoxElem::new(0.0, BoxSpread::new(10., 15., 17., 23., 42.))],
        )
        .horizontal();
        let dev_span = Span::new("Deviation", 15.5..=21.0);

        let avg_line = VLine::new("Average", 18.);

        let range = 32.;
        let min_x = 10. - range * 0.2;
        let max_x = 42. + range * 0.2;

        let available_width = ui.available_width();
        egui_plot::Plot::new("Help Plot")
            .legend(Legend::default())
            .allow_zoom(egui::Vec2b::new(true, false))
            .allow_drag(egui::Vec2b::new(true, false))
            .allow_scroll(egui::Vec2b::new(true, false))
            .auto_bounds(egui::Vec2b::new(true, false))
            .show_axes(egui::Vec2b::new(true, false))
            .center_y_axis(true)
            .include_x(min_x)
            .include_x(max_x)
            .default_y_bounds(-0.5, 1.5)
            .width(available_width)
            .show(ui, |plot_ui| {
                plot_ui.box_plot(plot);
                plot_ui.span(dev_span);
                plot_ui.vline(avg_line);
            });
    }
}

impl HelpSection for BenchmarkingStatsSection {
    fn section_name(&self) -> &str {
        "Reading benchmarking statistics"
    }

    fn section_ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("Benchmarking results will be plotted on a box graph. This help section will help you understand how to read this graph.");
        ui.add_space(10.);
        ui.label("First, the basics:");
        ui.label("- Average: how many instructions were run on average. This is represented with a green vertical line on the graph.");
        ui.add_space(5.);
        ui.label("The following elements can be found on the red box on the graph:");
        ui.label("- Minimum: this is the lowest amount of instructions used in the benchmark");
        ui.label("- Maximum: this is the highest amount of instructions used in the benchmark");
        ui.label("- Median: the exact middle value in the full list of instructions used. This number can differ from the average depending on the spread of instructions.");
        ui.label("- Quartile 1 and 3: the exact middle values of the first and second halves of the instructions list. This can be useful to visualize the spread of instructions.");
        ui.add_space(5.);
        ui.label("Finally:");
        ui.label("- Deviation: the average spread of instructions. This is visualized as a blue rectangle around the average line.");

        ui.add_space(10.);

        ui.label("Below is an example of a graph you can see after benchmarking.");
        ui.label("You can move around the graph by dragging with left click.");
        ui.label("You can zoom in a specific region by dragging with right click.");
        ui.label("You can also zoom in and out using Ctrl + Scroll Wheel.");
        ui.label("To reset the graph view to its default, double click anywhere on the graph.");
        self.draw_plot(ui);
    }
}
