use std::collections::HashMap;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const LICENSE: &str = "CC0-1.0";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalizableString(pub HashMap<String, String>);

impl LocalizableString {
    fn en(s: String) -> Self {
        let mut map = HashMap::new();
        map.insert("en".to_string(), s);
        Self(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Field {
    pub name: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<LocalizableString>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Schema {
    pub fields: Vec<Field>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
struct Tab {
    pub license: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub description: Option<LocalizableString>,
    pub schema: Schema,
    pub data: Vec<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axis {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<LocalizableString>,
    pub format: bool,
}

impl Default for Axis {
    fn default() -> Self {
        Axis {
            title: None,
            format: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chart {
    pub license: String,
    pub version: u64,
    pub r#type: String,
    #[serde(rename = "xAxis")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub x_axis: Option<Axis>,
    #[serde(rename = "yAxis")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub y_axis: Option<Axis>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<LocalizableString>,
}

// TODO: this should be an enum
fn convert_graph_chart_type(s: &str) -> &str {
    match s {
        "line" => "line",
        "bar" => "bar",
        "area" => "area",
        "pie" => "pie",
        _ => {
            println!("WARNING: Unknown chart type '{}', defaulting to 'line'.", s);
            "line"
        }
    }
}

fn convert_graph_types(s: &str) -> &str {
    match s {
        "integer" => "number",
        "number" => "number",
        "date" => "string",
        "string" => "string",
        _ => "string"
    }
}

// TODO: ty should be an enum
fn convert_graph_chart_value(value: &str, ty: &str) -> Value {
    if value.is_empty() {
        return Value::Null;
    }
    match ty {
        "number" => {
            if let Ok(num) = value.parse::<f64>() {
                Value::Number(serde_json::Number::from_f64(num).unwrap())
            } else {
                Value::String(value.to_string())
            }
        }
        "string" => Value::String(value.to_string()),
        _ => Value::String(value.to_string()),
    }
}

macro_rules! warn_unsupported_attr {
    ($tag:expr, $attr:expr) => {
        if $tag.contains_key($attr) {
            println!("WARNING: '{}' attribute is not supported by the chart extension.", $attr);
        }
    };
}

fn handle_graph_chart(tag: HashMap<String, Option<String>>) {
    warn_unsupported_attr!(tag, "colors");
    warn_unsupported_attr!(tag, "width");
    warn_unsupported_attr!(tag, "height");
    warn_unsupported_attr!(tag, "xScaleType");
    warn_unsupported_attr!(tag, "yScaleType");
    warn_unsupported_attr!(tag, "xAxisAngle");

    let chart_type = tag.get("type").cloned().unwrap_or_default().expect("'type' attribute not present");
    if chart_type == "pie" {
        unimplemented!()
    } else if chart_type.starts_with("stacked") {
        unimplemented!()
    }

    let x_type = convert_graph_types(&tag.get("xType").cloned().unwrap_or_default().unwrap_or("number".to_string())).to_string();
    let y_type = convert_graph_types(&tag.get("yType").cloned().unwrap_or_default().unwrap_or("number".to_string())).to_string();


    let mut fields = vec![Field {
        name: "x".to_string(),
        r#type: x_type.clone(),
        title: tag.get("xAxisTitle").cloned().unwrap_or_default().map(|s| LocalizableString::en(s))
    }];
    if tag.contains_key("y") {
        let y_field = Field {
            name: "y".to_string(),
            r#type: y_type.clone(),
            title: tag.get("yAxisTitle").cloned().unwrap_or_default().map(|s| LocalizableString::en(s))
        };
        fields.push(y_field);
    } else {
        let mut counter: u32 = 1;
        loop {
            let key = format!("y{counter}");
            if !tag.contains_key(&key) {
                break;
            }
            counter += 1;
        }
        assert_ne!(counter, 1);
        for i in 1..counter {
            let y_field = Field {
                name: format!("y{i}"),
                r#type: y_type.clone(),
                title: tag.get(&format!("y{i}Title")).cloned().unwrap_or_default().map(|s| LocalizableString::en(s))
            };
            fields.push(y_field);
        }
    }
    let x_values: Vec<_> = tag.get("x").cloned().unwrap_or_default().expect("'x' attribute not present").split(",").map(|s| s.trim()).map(|s| convert_graph_chart_value(s, &x_type)).collect();
    let y_values: Vec<Vec<_>> = if tag.contains_key("y") {
         vec![tag.get("y").cloned().unwrap_or_default().expect("'y' attribute not present").split(",").map(|s| s.trim()).map(|s| convert_graph_chart_value(s, &y_type)).collect()]
    } else {
        let mut values = Vec::new();
        let mut counter: u32 = 1;
        loop {
            let key = format!("y{counter}");
            if !tag.contains_key(&key) {
                break;
            }
            let y_values = tag.get(&key).cloned().unwrap_or_default().expect(&format!("'{}' attribute not present", key));
            let values_for_y: Vec<_> = y_values.split(",").map(|s| s.trim()).map(|s| convert_graph_chart_value(s, &y_type)).collect();
            values.push(values_for_y);
            counter += 1;
        }
        values
    };
    let tab = Tab {
        license: LICENSE.to_string(),
        description: tag.get("description").cloned().unwrap_or_default().map(|s| LocalizableString::en(s)),
        schema: Schema { fields },
        data: x_values.into_iter().enumerate().map(|(count, v)| {
            let mut out = vec![v];
            for y_value in &y_values {
                if count < y_value.len() {
                    out.push(y_value[count].clone());
                } else {
                    out.push(Value::Null);
                }
            }
            out
        }).collect()
    };
    let stringified = serde_json::to_string_pretty(&tab).expect("Failed to serialize tab");
    let prompt = inquire::Text::new("Please enter a name for the chart:").prompt().unwrap();
    let file_name = format!("{}", prompt.replace(' ', "_"));
    let tab_file_name = format!("{}.tab", file_name);
    std::fs::write(&tab_file_name, stringified).expect("Failed to write tab file");
    let chart = Chart {
        license: LICENSE.to_string(),
        version: 1,
        r#type: convert_graph_chart_type(&chart_type).to_string(),
        x_axis: None,
        y_axis: None,
        source: tab_file_name,
        title: Some(tag.get("title").cloned().unwrap_or_default().map(|s| LocalizableString::en(s)).unwrap_or(LocalizableString::en(prompt.clone()))),
    };
    let chart_stringified = serde_json::to_string_pretty(&chart).expect("Failed to serialize chart");
    let chart_file_name = format!("{}.chart", file_name);
    std::fs::write(&chart_file_name, chart_stringified).expect("Failed to write chart file");
}

#[derive(clap::Parser)]
struct Args {
    /// Article
    article: String,
    /// Which wiki to use
    #[clap(long, default_value = "en.wikipedia.org")]
    wiki: String,
}

#[derive(Serialize, Deserialize)]
struct Template {
    pub name: String,
    pub params: HashMap<String, Option<String>>,
}

#[derive(Serialize, Deserialize)]
struct OutRoot {
    pub data: Vec<Template>,
}

fn build_url(article: &str, wiki: &str) -> String {
    // Replace spaces with underscores
    let article = article.replace(' ', "_");
    format!("https://{}/w/index.php?action=raw&title={}", wiki, article)
}

fn main() {
    let args = Args::parse();
    // Download the article
    let req = ureq::get(build_url(&args.article, &args.wiki));
    let response = req.call().expect("Failed to fetch article");
    if !response.status().is_success() {
        eprintln!("Error fetching article: {}", response.status());
        std::process::exit(1);
    }
    let mut body = response.into_body();
    let content = body.read_to_string().expect("Failed to read response body");
    // Write the content to in.txt
    std::fs::write("in.txt", content).expect("Failed to write in.txt");
    // Call main.py
    let output = std::process::Command::new("uv")
        .arg("run")
        .arg("main.py")
        .output()
        .expect("Failed to execute main.py");
    if !output.status.success() {
        eprintln!("Error running main.py: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    // Read out.json
    let input = std::fs::read_to_string("out.json").expect("Failed to read out.json");
    let templates: OutRoot = serde_json::from_str(&input).expect("Failed to parse JSON");
    for parsed in templates.data {
        match &*parsed.name {
            "Graph:Chart" | "GraphChart" => {
                handle_graph_chart(parsed.params)
            }
            _ => {
                // println!("Unknown tag: {}", parsed.name);
            }
        }
    }
}
