use crate::schema::chart::Chart;
use crate::schema::tab::Tab;

mod graph_chart;
mod graph_tag;
mod pie_chart;

pub use graph_chart::generate as gen_graph_chart;

pub struct ConversionOutput {
    pub chart: Chart,
    pub tab: Tab,
}
