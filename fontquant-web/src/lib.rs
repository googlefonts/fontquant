#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::str::FromStr;

use fontquant_lib::{MetricValue, Results};
use read_fonts::types::Tag;
use read_fonts::FontRef;
use serde_json::{Map, Value};
use skrifa::setting::Setting;
use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn parse_location(s: &str) -> Result<Vec<Setting<f32>>, String> {
    if s.is_empty() {
        return Ok(vec![]);
    }
    let mut settings = Vec::new();
    for setting in s.split(',') {
        let parts: Vec<&str> = setting.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid setting format: {s}"));
        }
        let value: f32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid value for setting: {s}"))?;
        let tag = Tag::from_str(parts[0]).map_err(|_| format!("Invalid tag: {}", parts[0]))?;
        settings.push(Setting::new(tag, value));
    }
    Ok(settings)
}

fn metric_value_to_json(mv: &MetricValue) -> Value {
    match mv {
        MetricValue::Metric(f)
        | MetricValue::Percentage(f)
        | MetricValue::Angle(f)
        | MetricValue::PerMille(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        MetricValue::String(s) => Value::String(s.clone()),
        MetricValue::List(l) => Value::Array(l.iter().cloned().map(Value::String).collect()),
        MetricValue::Dictionary(d) => Value::Object(
            d.iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect(),
        ),
        MetricValue::Boolean(b) => Value::Bool(*b),
        MetricValue::Integer(i) => Value::Number((*i).into()),
    }
}

fn results_to_json(results: &Results) -> Value {
    let mut map = Map::new();
    for (name, (_key, value)) in results.iter() {
        map.insert(name.clone(), metric_value_to_json(value));
    }
    Value::Object(map)
}

#[wasm_bindgen]
pub fn run(font_data: &[u8], location: Option<String>) -> Result<String, JsValue> {
    let font = FontRef::new(font_data).map_err(|e| JsValue::from(e.to_string()))?;
    let loc = parse_location(location.as_deref().unwrap_or("")).map_err(JsValue::from)?;
    let results = fontquant_lib::run(&font, &loc).map_err(|e| JsValue::from(e.to_string()))?;
    serde_json::to_string(&results_to_json(&results)).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen]
pub fn get_parametric(font_data: &[u8], location: Option<String>) -> Result<String, JsValue> {
    let font = FontRef::new(font_data).map_err(|e| JsValue::from(e.to_string()))?;
    let loc = parse_location(location.as_deref().unwrap_or("")).map_err(JsValue::from)?;
    let mut results = Results::new();
    fontquant_lib::quantifiers::parametric::get_parametric(&font, &loc, &mut results)
        .map_err(|e| JsValue::from(e.to_string()))?;
    serde_json::to_string(&results_to_json(&results)).map_err(|e| JsValue::from(e.to_string()))
}
