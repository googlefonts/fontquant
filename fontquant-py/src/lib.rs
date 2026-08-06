#![deny(clippy::unwrap_used, clippy::expect_used)]
use std::collections::BTreeMap;

use pyo3::{exceptions::PyRuntimeError, prelude::*, IntoPyObjectExt};
use read_fonts::FontRef;

use fontquant_lib::{MetricValue, Results};

fn pythonize_metric_value(metric_value: &MetricValue, py: Python<'_>) -> Result<Py<PyAny>, PyErr> {
    match metric_value {
        MetricValue::Metric(f) => f.into_py_any(py),
        MetricValue::Percentage(f) => f.into_py_any(py),
        MetricValue::String(s) => s.into_py_any(py),
        MetricValue::Dictionary(d) => d.into_py_any(py),
        MetricValue::List(s) => s.into_py_any(py),
        MetricValue::MetricList(s) => s.into_py_any(py),
        MetricValue::Boolean(b) => b.into_py_any(py),
        MetricValue::Angle(a) => a.into_py_any(py),
        MetricValue::PerMille(p) => p.into_py_any(py),
        MetricValue::Integer(i) => i.into_py_any(py),
    }
}

fn pythonize_results(results: Results, py: Python<'_>) -> Result<Bound<'_, PyAny>, PyErr> {
    let local: BTreeMap<String, _> = results
        .iter()
        .map(|(label, (_metric_key, metric_value))| {
            Ok((label.to_string(), pythonize_metric_value(metric_value, py)?))
        })
        .collect::<Result<BTreeMap<_, _>, PyErr>>()?;
    local.into_bound_py_any(py)
}

#[pyfunction]
fn get_parametric<'a>(py: Python<'a>, font_file: &str) -> Result<Bound<'a, PyAny>, PyErr> {
    let mut results = Results::new();
    let font_file = std::fs::read(font_file)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to read font file: {e}")))?;
    let font_file = FontRef::new(&font_file)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to parse font file: {e}")))?;
    fontquant_lib::quantifiers::parametric::get_parametric(&font_file, &[], &mut results)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("{e}")))?;
    pythonize_results(results, py)
}

#[pyfunction]
fn run<'a>(py: Python<'a>, font_file: &str) -> Result<Bound<'a, PyAny>, PyErr> {
    let font_file = std::fs::read(font_file)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to read font file: {e}")))?;
    let font_file = FontRef::new(&font_file)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to parse font file: {e}")))?;
    let results = fontquant_lib::run(&font_file, &[])
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("{e}")))?;
    pythonize_results(results, py)
}

#[pymodule(name = "_fontquant")]
fn fontquant(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_parametric, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)
}
