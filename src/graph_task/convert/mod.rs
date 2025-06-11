use crate::graph_task::schema::{chart::Chart, tab::Tab};

mod graph_chart;
mod graph_tag;
mod pie_chart;

pub use graph_chart::generate as gen_graph_chart;

pub struct ConversionOutput {
    pub chart: Chart,
    pub tab: Tab,
}
