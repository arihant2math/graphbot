use std::collections::HashMap;

use anyhow::{anyhow, bail};
use serde_json::{Number, Value};
use tracing::warn;

use crate::{
    TAB_EXT,
    graph_task::{
        convert::ConversionOutput,
        schema::{
            LocalizableString,
            chart::{Axis, Chart, ChartType},
            tab::{Field, Tab},
        },
    },
};

const LICENSE: &str = "CC-BY-SA-4.0";

fn convert_graph_chart_type(s: &str) -> ChartType {
    match &*s.to_ascii_lowercase() {
        "line" => ChartType::Line,
        "bar" | "rect" => ChartType::Bar,
        "area" => ChartType::Area,
        "pie" => ChartType::Pie,
        "stackedrect" => ChartType::Area,
        _ => {
            warn!("Unknown chart type '{s}', defaulting to 'line'.");
            ChartType::Line
        }
    }
}

fn convert_graph_types(s: &str) -> &str {
    match s {
        "integer" | "number" => "number",
        "date" | "string" => "string",
        _ => "string",
    }
}

fn parse_number(value: &str) -> Option<Number> {
    // Replace the unicode minus sign with a regular hyphen
    // This was 20 minutes of debugging, because the minus sign was not being parsed
    // correctly
    let mut value = value.replace("\u{2212}", "-");
    value = value.trim().to_string();

    if let Ok(i) = value.parse::<i128>() {
        return Number::from_i128(i);
    } else if let Ok(f) = value.parse::<f64>() {
        return Number::from_f64(f);
    }
    None
}

#[cfg(test)]
mod number_parse_tests {
    use serde_json::Number;

    use super::parse_number;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("42"), Number::from_i128(42i128));
        assert_eq!(parse_number("-42"), Number::from_i128(-42i128));
        assert_eq!(parse_number("3.14"), Number::from_f64(3.14));
        assert_eq!(parse_number("-3.14"), Number::from_f64(-3.14));
        assert_eq!(parse_number("0"), Number::from_i128(0i128));
        assert_eq!(parse_number("1e3"), Number::from_f64(1000.0));
        assert_eq!(parse_number("1.5e-2"), Number::from_f64(0.015));
        assert_eq!(parse_number("not a number"), None);
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number(".42"), Number::from_f64(0.42));
    }
}

// TODO: ty should be an enum
fn convert_graph_chart_value(value: &str, ty: &str) -> Value {
    if value.is_empty() {
        return Value::Null;
    }
    match ty {
        "number" => {
            if let Some(num) = parse_number(value) {
                Value::Number(num)
            } else {
                Value::String(value.to_string())
            }
        }
        "string" => Value::String(value.to_string()),
        _ => Value::String(value.to_string()),
    }
}

pub fn generate(
    name: &str,
    tag: &HashMap<String, Option<String>>,
    source_url: &str,
) -> anyhow::Result<ConversionOutput> {
    let supported_attrs = [
        "type",
        "xType",
        "yType",
        "xAxisTitle",
        "yAxisTitle",
        "title",
        "description",
        "x",
        "y",
        "y1",
        "y2",
        "y3",
        "y4",
        "y5",
        "y6",
        "y7",
        "y8",
        "y9",
        "xTitle",
        "y1Title",
        "y2Title",
        "y3Title",
        "y4Title",
        "y5Title",
        "y6Title",
        "y7Title",
        "y8Title",
        "y9Title",
    ];
    for attr in tag.keys() {
        if !supported_attrs.contains(&attr.as_str()) {
            warn!("Unsupported attribute '{}' in graph chart tag.", attr);
        }
    }

    let chart_type = tag
        .get("type")
        .cloned()
        .flatten()
        .ok_or_else(|| anyhow!("'type' attribute not present"))?;
    if chart_type == "pie" {
        let chart = Chart {
            license: LICENSE.to_string(),
            r#type: convert_graph_chart_type(&chart_type),
            x_axis: None,
            y_axis: None,
            source: tab_file_name,
            title: Some(
                tag.get("title")
                    .cloned()
                    .unwrap_or_default()
                    .map(LocalizableString::en)
                    .unwrap_or(LocalizableString::en(name.to_string())),
            ),
            ..Default::default()
        };
        bail!("Pie charts are not supported yet");
        // Ok(ConversionOutput {
        //     chart,
        //     tab: gen_pie_tab(tag, source_url)?,
        // })
    } else if chart_type.starts_with("stacked") && &chart_type != "stackedrect" {
        bail!("Non-rect stacked charts are not supported yet by the chart extension");
    }

    let tab_file_name = format!("{name}{TAB_EXT}");

    macro_rules! gen_axis {
        ($tag:expr, $name:expr) => {
            match $tag.get($name) {
                Some(Some(value)) => Some(Axis {
                    title: Some(LocalizableString::en(value.clone())),
                    ..Axis::default()
                }),
                _ => None,
            }
        };
    }

    let chart = Chart {
        license: LICENSE.to_string(),
        r#type: convert_graph_chart_type(&chart_type),
        x_axis: gen_axis!(tag, "xAxisTitle"),
        y_axis: gen_axis!(tag, "yAxisTitle"),
        source: tab_file_name,
        title: Some(
            tag.get("title")
                .cloned()
                .unwrap_or_default()
                .map(LocalizableString::en)
                .unwrap_or(LocalizableString::en(name.to_string())),
        ),
        ..Default::default()
    };
    Ok(ConversionOutput {
        chart,
        tab: gen_tab(tag, source_url)?,
    })
}

fn gen_tab(tag: &HashMap<String, Option<String>>, source_url: &str) -> anyhow::Result<Tab> {
    let x_type = convert_graph_types(
        &tag.get("xType")
            .cloned()
            .unwrap_or_default()
            .unwrap_or("number".to_string()), // TODO: Need to check actual type
    )
    .to_string();
    let y_type = convert_graph_types(
        &tag.get("yType")
            .cloned()
            .unwrap_or_default()
            .unwrap_or("number".to_string()),
    )
    .to_string();

    let x_values: Vec<_> = tag
        .get("x")
        .cloned()
        .flatten()
        .ok_or_else(|| anyhow!("'x' attribute not present"))?
        .split(',')
        .map(str::trim)
        .map(|s| convert_graph_chart_value(s, &x_type))
        .collect();
    let y_values: Vec<Vec<_>> = if tag.contains_key("y") {
        vec![
            tag.get("y")
                .cloned()
                .flatten()
                .ok_or_else(|| anyhow!("'y' attribute not present"))?
                .split(',')
                .map(str::trim)
                .map(|s| convert_graph_chart_value(s, &y_type))
                .collect(),
        ]
    } else {
        let mut values = Vec::new();
        let mut counter: u32 = 1;
        loop {
            let key = format!("y{counter}");
            if !tag.contains_key(&key) {
                break;
            }
            let y_values = tag
                .get(&key)
                .cloned()
                .unwrap_or_default()
                .ok_or_else(|| anyhow!("'{}' attribute not present", key))?;
            let values_for_y: Vec<_> = y_values
                .split(',')
                .map(str::trim)
                .map(|s| convert_graph_chart_value(s, &y_type))
                .collect();
            values.push(values_for_y);
            counter += 1;
        }
        values
    };
    let table = Tab {
        license: LICENSE.to_string(),
        sources: Some(source_url.to_string()),
        description: tag
            .get("description")
            .cloned()
            .unwrap_or_default()
            .map(LocalizableString::en),
        schema: gen_fields(tag, &x_type, &y_type).into(),
        data: x_values
            .into_iter()
            .enumerate()
            .map(|(count, v)| {
                let mut out = vec![v];
                for y_value in &y_values {
                    if count < y_value.len() {
                        out.push(y_value[count].clone());
                    } else {
                        out.push(Value::Null);
                    }
                }
                out
            })
            .collect(),
        ..Default::default()
    };
    Ok(table)
}

fn gen_fields(tag: &HashMap<String, Option<String>>, x_type: &str, y_type: &str) -> Vec<Field> {
    let mut fields = vec![Field {
        name: "x".to_string(),
        r#type: x_type.to_string(),
        title: tag
            .get("xAxisTitle")
            .cloned()
            .unwrap_or_default()
            .map(LocalizableString::en),
    }];
    if tag.contains_key("y") {
        let y_field = Field {
            name: "y".to_string(),
            r#type: y_type.to_string(),
            title: tag
                .get("yAxisTitle")
                .cloned()
                .flatten()
                .map(LocalizableString::en),
        };
        fields.push(y_field);
    } else if tag.contains_key("y1") && !tag.contains_key("y2") {
        let y_field = Field {
            name: "y1".to_string(),
            r#type: y_type.to_string(),
            title: tag
                .get("y1Title")
                .cloned()
                .flatten()
                // yAxisTitle is a fallback
                .or_else(|| tag.get("yAxisTitle").cloned().flatten())
                .map(LocalizableString::en),
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
                r#type: y_type.to_string(),
                title: tag
                    .get(&format!("y{i}Title"))
                    .cloned()
                    .unwrap_or_default()
                    .map(LocalizableString::en),
            };
            fields.push(y_field);
        }
    }
    fields
}
