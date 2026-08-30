use std::{collections::HashMap, fmt::Display};

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
            tab::{Field, Schema, Tab},
        },
    },
};

const LICENSE: &str = "CC-BY-SA-4.0";

fn convert_graph_chart_type(s: &str) -> ChartType {
    match &*s.to_ascii_lowercase() {
        "line" => ChartType::Line,
        "bar" | "rect" => ChartType::Bar,
        "area" | "stackedrect" => ChartType::Area,
        "pie" => ChartType::Pie,
        _ => {
            warn!("Unknown chart type '{s}', defaulting to 'line'.");
            ChartType::Line
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ValueType {
    Number,
    String,
    Bool,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Number => write!(f, "number"),
            ValueType::String => write!(f, "string"),
            ValueType::Bool => write!(f, "boolean"),
        }
    }
}

fn convert_graph_types(s: &str) -> ValueType {
    match s.to_lowercase().as_str() {
        "integer" | "number" => ValueType::Number,
        "date" | "string" => ValueType::String,
        _ => {
            warn!("Unknown type '{s}', defaulting to 'number'.");
            ValueType::Number
        }
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
mod tests {
    use std::collections::HashMap;

    use serde_json::Number;

    use super::{generate, parse_number};
    use crate::graph_task::convert::ConversionOutput;

    #[test]
    #[allow(clippy::approx_constant)]
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

    #[test]
    fn graph_without_y_values_returns_error() {
        let tag = HashMap::from([("x".to_string(), Some("1,2,3".to_string()))]);

        let result = generate("No y values", &tag, "https://example.com");

        assert_eq!(
            result.err().map(|error| error.to_string()).as_deref(),
            Some("Neither 'y' nor 'y1' present")
        );
    }

    #[test]
    fn graph_with_external_table_uses_existing_data() {
        let tag = HashMap::from([
            (
                "table".to_string(),
                Some("Data:ncei.noaa.gov/weather/Honolulu.tab".to_string()),
            ),
            ("x".to_string(), Some("date".to_string())),
            (
                "title".to_string(),
                Some("Honolulu monthly weather statistics".to_string()),
            ),
        ]);

        let output = generate("HonoluluWeather", &tag, "https://example.com").unwrap();

        match output {
            ConversionOutput::ExistingData {
                chart,
                tab_file_name,
                x_field,
            } => {
                assert_eq!(tab_file_name, "ncei.noaa.gov/weather/Honolulu.tab");
                assert_eq!(chart.source, tab_file_name);
                assert_eq!(x_field.as_deref(), Some("date"));
            }
            ConversionOutput::GeneratedData { .. } => panic!("expected existing table data"),
        }
    }

    #[test]
    fn external_table_name_allows_colons_and_rejects_fragments() {
        let with_colon = HashMap::from([(
            "table".to_string(),
            Some("Data:weather:Honolulu".to_string()),
        )]);
        let output = generate("HonoluluWeather", &with_colon, "https://example.com").unwrap();
        assert!(matches!(
            output,
            ConversionOutput::ExistingData { tab_file_name, .. }
                if tab_file_name == "weather:Honolulu.tab"
        ));

        let with_fragment = HashMap::from([(
            "table".to_string(),
            Some("weather/Honolulu.tab#data".to_string()),
        )]);
        let error = generate("HonoluluWeather", &with_fragment, "https://example.com").unwrap_err();
        assert!(error.to_string().contains("section fragment"));
    }
}

fn convert_graph_chart_value(value: &str, ty: ValueType) -> Value {
    if value.is_empty() {
        return Value::Null;
    }
    match ty {
        ValueType::Number => {
            if let Some(num) = parse_number(value) {
                Value::Number(num)
            } else {
                Value::String(value.to_string())
            }
        }
        ValueType::String => Value::String(value.to_string()),
        ValueType::Bool => {
            let lower = value.to_lowercase();
            if lower == "true" || lower == "1" {
                Value::Bool(true)
            } else if lower == "false" || lower == "0" {
                Value::Bool(false)
            } else {
                Value::String(value.to_string())
            }
        }
    }
}

pub fn generate(
    name: &str,
    tag: &HashMap<String, Option<String>>,
    source_url: &str,
) -> anyhow::Result<ConversionOutput> {
    let supported_attrs = [
        "type",
        "table",
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
    let mut unsupported_attrs: Vec<_> = tag
        .keys()
        .filter(|attr| !supported_attrs.contains(&attr.as_str()))
        .cloned()
        .collect();
    unsupported_attrs.sort();
    if !unsupported_attrs.is_empty() {
        warn!(
            unsupported_attributes = ?unsupported_attrs,
            "Graph chart contains unsupported attributes"
        );
    }

    let chart_type = tag
        .get("type")
        .cloned()
        .flatten()
        .unwrap_or("line".to_string());
    if chart_type.starts_with("stacked") && chart_type != "stackedrect" {
        bail!("Non-rect stacked charts are not supported yet by the chart extension");
    }

    let existing_tab = tag
        .get("table")
        .map(|value| {
            value
                .as_deref()
                .ok_or_else(|| anyhow!("'table' attribute has no value"))
                .and_then(normalize_tab_file_name)
        })
        .transpose()?;
    if existing_tab.is_some()
        && tag.keys().any(|key| {
            key == "y"
                || key
                    .strip_prefix('y')
                    .is_some_and(|v| v.parse::<u32>().is_ok())
        })
    {
        bail!("External table data cannot be combined with inline y values");
    }
    let tab_file_name = existing_tab
        .clone()
        .unwrap_or_else(|| format!("{name}{TAB_EXT}"));

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
        x_axis: (chart_type != "pie")
            .then(|| gen_axis!(tag, "xAxisTitle"))
            .flatten(),
        y_axis: (chart_type != "pie")
            .then(|| gen_axis!(tag, "yAxisTitle"))
            .flatten(),
        source: tab_file_name.clone(),
        title: Some(
            tag.get("title")
                .cloned()
                .unwrap_or_default()
                .map(LocalizableString::en)
                .unwrap_or(LocalizableString::en(name.to_string())),
        ),
        ..Default::default()
    };

    if let Some(tab_file_name) = existing_tab {
        let x_field = tag.get("x").cloned().flatten();
        if x_field.as_deref().is_some_and(|field| field.contains(',')) {
            bail!("External table x must name one field, not inline values");
        }
        return Ok(ConversionOutput::ExistingData {
            chart,
            tab_file_name,
            x_field,
        });
    }

    let tab = if chart_type == "pie" {
        gen_pie_tab(tag, source_url)?
    } else {
        gen_tab(tag, source_url)?
    };
    Ok(ConversionOutput::GeneratedData { chart, tab })
}

fn normalize_tab_file_name(value: &str) -> anyhow::Result<String> {
    let mut value = value.trim();
    if value
        .get(.."Data:".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Data:"))
    {
        value = &value["Data:".len()..];
    }
    let value = value.trim();
    if value.is_empty() {
        bail!("'table' attribute is empty");
    }
    if value.contains('#') {
        bail!("External table must not contain a section fragment: {value}");
    }

    if value.to_ascii_lowercase().ends_with(TAB_EXT) {
        Ok(value.to_string())
    } else {
        Ok(format!("{value}{TAB_EXT}"))
    }
}

fn detect_type(s: &str) -> Option<ValueType> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let strings: Vec<_> = s.split(',').map(str::trim).collect();
    let mut number = true;
    for item in &strings {
        if parse_number(item).is_none() {
            number = false;
            break;
        }
    }
    if number {
        Some(ValueType::Number)
    } else {
        Some(ValueType::String)
    }
}

fn gen_tab(tag: &HashMap<String, Option<String>>, source_url: &str) -> anyhow::Result<Tab> {
    let x_type = if let Some(ty) = tag.get("xType").cloned().unwrap_or_default() {
        convert_graph_types(&ty)
    } else {
        detect_type(
            &tag.get("x")
                .cloned()
                .flatten()
                .ok_or_else(|| anyhow!("'x' attribute not present"))?,
        )
        .ok_or_else(|| anyhow!("Cannot infer xType"))?
    };
    let y_type = if let Some(ty) = tag.get("yType").cloned().unwrap_or_default() {
        convert_graph_types(&ty)
    } else {
        ValueType::Number
    };

    let x_values: Vec<_> = tag
        .get("x")
        .cloned()
        .flatten()
        .ok_or_else(|| anyhow!("'x' attribute not present"))?
        .split(',')
        .map(str::trim)
        .map(|s| convert_graph_chart_value(s, x_type))
        .collect();
    let y_values: Vec<Vec<_>> = if tag.contains_key("y") {
        vec![
            tag.get("y")
                .cloned()
                .flatten()
                .ok_or_else(|| anyhow!("'y' attribute not present"))?
                .split(',')
                .map(str::trim)
                .map(|s| convert_graph_chart_value(s, y_type))
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
                .map(|s| convert_graph_chart_value(s, y_type))
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
        schema: gen_fields(tag, x_type, y_type)?.into(),
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

fn gen_fields(
    tag: &HashMap<String, Option<String>>,
    x_type: ValueType,
    y_type: ValueType,
) -> anyhow::Result<Vec<Field>> {
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
        if counter == 1 {
            bail!("Neither 'y' nor 'y1' present");
        }
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
    Ok(fields)
}

fn gen_pie_tab(tag: &HashMap<String, Option<String>>, source_url: &str) -> anyhow::Result<Tab> {
    let names: Vec<_> = tag
        .get("x")
        .cloned()
        .flatten()
        .ok_or_else(|| anyhow!("'x' attribute not present"))?
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .collect();
    if tag.contains_key("y2") {
        bail!("Pie charts cannot have y2");
    }
    let y_key = if tag.contains_key("y") {
        "y"
    } else if tag.contains_key("y1") {
        "y1"
    } else {
        bail!("Neither 'y' nor 'y1' present");
    };
    let y_values: Vec<_> = tag
        .get(y_key)
        .cloned()
        .flatten()
        .ok_or_else(|| anyhow!("'{y_key}' attribute not present"))?
        .split(',')
        .map(str::trim)
        .map(|s| convert_graph_chart_value(s, ValueType::Number))
        .collect();
    let table = Tab {
        license: LICENSE.to_string(),
        sources: Some(source_url.to_string()),
        description: tag
            .get("description")
            .cloned()
            .unwrap_or_default()
            .map(LocalizableString::en),
        schema: Schema {
            fields: names
                .into_iter()
                .enumerate()
                .map(|(count, value)| Field {
                    name: format!("item{count}"),
                    r#type: "number".to_string(),
                    title: Some(LocalizableString::en(value)),
                })
                .collect(),
        },
        data: vec![y_values],
        ..Default::default()
    };
    Ok(table)
}
