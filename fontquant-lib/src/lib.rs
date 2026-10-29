#![deny(clippy::unwrap_used, clippy::expect_used)]
use quantifiers::ALL_QUANTIFIERS;

use crate::error::FontquantError;
use std::collections::{BTreeMap, HashMap};

mod bezglyph;
mod error;
mod helpers;
mod monkeypatching;
pub mod quantifiers;

#[macro_export]
macro_rules! quantifier {
    ($ident:ident, $name:expr, $description:expr, $example_value:expr) => {
        pub static $ident: std::sync::LazyLock<$crate::MetricKey> =
            std::sync::LazyLock::new(|| $crate::MetricKey {
                name: $name.to_string(),
                description: $description.to_string(),
                example_value: $example_value,
            });
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    /// A floating point number representing a metric value.
    Metric(f64),
    /// A percentage value, typically between 0.0 and 1.0.
    Percentage(f64),
    /// A string value, often used for descriptive metrics.
    String(String),
    /// A list of strings, often used for metrics that return multiple values.
    List(Vec<String>),
    /// A list of floating point numbers
    MetricList(Vec<f64>),
    /// Dictionary of string key-value pairs, often used for metrics that return multiple named values.
    Dictionary(HashMap<String, String>),
    /// A boolean value, typically true or false.
    Boolean(bool),
    /// A value representing an angle in degrees. Store in degrees.
    Angle(f64),
    /// A value representing a length in em units.
    PerMille(f64),
    /// A value representing a length in indeterminate units.
    Integer(i32),
}

impl MetricValue {
    pub fn as_boolean(&self) -> Option<bool> {
        if let MetricValue::Boolean(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

impl TryInto<f64> for MetricValue {
    type Error = &'static str;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            MetricValue::Metric(value) => Ok(value),
            MetricValue::Percentage(value) => Ok(value),
            MetricValue::PerMille(value) => Ok(value),
            _ => Err("Cannot convert to f64"),
        }
    }
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Metric(value) => write!(f, "{:.2}", value),
            MetricValue::Percentage(value) => write!(f, "{:.2}%", value * 100.0),
            MetricValue::String(value) => write!(f, "{}", value),
            MetricValue::List(values) => write!(f, "{}", values.join(", ")),
            MetricValue::MetricList(values) => {
                let formatted: Vec<String> = values.iter().map(|v| format!("{:.2}", v)).collect();
                write!(f, "{}", formatted.join(", "))
            }
            MetricValue::Dictionary(values) => {
                let formatted: Vec<String> = values
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", formatted.join(", "))
            }
            MetricValue::Boolean(value) => write!(f, "{}", value),
            MetricValue::Angle(value) => write!(f, "{:.2}", value),
            MetricValue::PerMille(value) => write!(f, "{:.0}", value),
            MetricValue::Integer(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricKey {
    pub name: String,
    pub description: String,
    pub example_value: MetricValue,
}

pub type Metric = (&'static MetricKey, MetricValue);

#[derive(Debug, Clone, Default)]
pub struct Results(BTreeMap<String, Metric>);
impl Results {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_metric(&mut self, metric: &'static MetricKey, value: MetricValue) {
        self.0.insert(metric.name.clone(), (metric, value));
    }

    pub fn get(&self, name: &str) -> Option<&Metric> {
        self.0.get(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Metric)> {
        self.0.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    pub fn filter(&self, strings: &[String]) -> Self {
        if strings.is_empty() {
            return self.clone();
        }
        let mut filtered = Results::new();
        for (name, metric) in self.iter() {
            if strings.iter().any(|s| name.contains(s)) {
                filtered.add_metric(metric.0, metric.1.clone());
            }
        }
        filtered
    }
}

pub fn run(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
    only_run: &[String],
) -> Result<Results, FontquantError> {
    let mut results = Results::new();
    // We don't need this yet, but when we seriously start playing with variations,
    // then it will be useful:

    // let axes = font.axes();
    // let mut norm_location = vec![NormalizedCoord::default(); axes.len()];
    // axes.location_to_slice(location, &mut norm_location);

    for (metric, quantifier) in ALL_QUANTIFIERS.iter() {
        if !only_run.is_empty() && !only_run.iter().any(|s| metric.name.contains(s)) {
            continue;
        }
        if results.get(&metric.name).is_some() {
            // May have already been filled in by an earlier function
            continue;
        }
        quantifier(font, location, &mut results)?;
    }
    Ok(results)
}
