#![deny(clippy::unwrap_used, clippy::expect_used)]
use quantifiers::ALL_QUANTIFIERS;

use crate::error::FontquantError;
use fontations::skrifa;
use std::collections::BTreeMap;

mod bezglyph;
mod error;
mod helpers;
mod monkeypatching;
mod quantifiers;

#[macro_export]
macro_rules! quantifier {
    ($ident:ident, $name:expr, $description:expr, $example_value:expr) => {
        static $ident: std::sync::LazyLock<$crate::MetricKey> =
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
    /// A boolean value, typically true or false.
    Boolean(bool),
    /// A value representing an angle in degrees.
    Angle(f64),
    /// A value representing a length in em units.
    PerMille(f64),
    /// A value representing a length in indeterminate units.
    Integer(i32),
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
}

pub fn run(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
) -> Result<Results, FontquantError> {
    let mut results = Results::new();
    for metric in ALL_QUANTIFIERS.iter() {
        metric(font, location, &mut results)?;
    }
    Ok(results)
}
