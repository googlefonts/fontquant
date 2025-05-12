#![deny(clippy::unwrap_used, clippy::expect_used)]
use quantifiers::ALL_QUANTIFIERS;

use crate::error::FontquantError;
use std::collections::HashMap;

mod bezglyph;
mod error;
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
    Metric(f64),
    Percentage(f64),
    String(String),
    Boolean(bool),
    Angle(f64),
    PerMille(f64),
    Integer(i32),
}

#[derive(Debug, Clone)]
pub struct MetricKey {
    pub name: String,
    pub description: String,
    pub example_value: MetricValue,
}

#[derive(Debug, Clone, Default)]
pub struct Results(HashMap<String, (&'static MetricKey, MetricValue)>);
impl Results {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_metric(&mut self, metric: &'static MetricKey, value: MetricValue) {
        self.0.insert(metric.name.clone(), (metric, value));
    }

    pub fn get(&self, name: &str) -> Option<&(&'static MetricKey, MetricValue)> {
        self.0.get(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &(&'static MetricKey, MetricValue))> {
        self.0.iter()
    }
}

pub trait MetricGatherer {
    fn gather_from_font(
        font: &skrifa::FontRef,
        location: &[skrifa::setting::VariationSetting],
        results: &mut Results,
    ) -> Result<(), FontquantError>;
}

pub fn run(
    font: &skrifa::FontRef,
    location: &[skrifa::setting::VariationSetting],
) -> Result<Results, FontquantError> {
    let mut results = Results::new();
    #[allow(clippy::single_element_loop)]
    for metric in ALL_QUANTIFIERS.iter() {
        metric(font, location, &mut results)?;
    }
    Ok(results)
}
