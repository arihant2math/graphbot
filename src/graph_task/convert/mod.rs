use crate::graph_task::schema::{chart::Chart, tab::Tab};

mod graph_chart;
mod graph_tag;
mod pie_chart;

pub use graph_chart::generate as gen_graph_chart;

#[derive(Debug)]
pub enum ConversionOutput {
    GeneratedData {
        chart: Chart,
        tab: Tab,
    },
    ExistingData {
        chart: Chart,
        tab_file_name: String,
        x_field: Option<String>,
    },
}
