use pyo3::{exceptions::PyValueError, prelude::*};

/// Return a deterministic JSON report, or raise ValueError on any invalid input.
#[pyfunction]
#[pyo3(signature = (source, format="oath"))]
fn verify(source: &str, format: &str) -> PyResult<String> {
    let report = oathkeeper_core::verify_text(source, format)
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    serde_json::to_string(&report).map_err(|error| PyValueError::new_err(error.to_string()))
}

#[pyfunction]
fn verify_case(module_json: &str, case_json: &str) -> PyResult<String> {
    let bundle = oathkeeper_core::assemble_case(module_json, case_json)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let report =
        oathkeeper_core::verify(&bundle).map_err(|e| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&report).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Compile an authoring source graph into IR, a transitive content lock, and source maps.
#[pyfunction]
fn compile_file(path: &str) -> PyResult<String> {
    let output =
        oathkeeper_core::compile_file(path).map_err(|e| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&output).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn verify_file(path: &str) -> PyResult<String> {
    let report = oathkeeper_core::compile_file(path)
        .and_then(|c| c.verify())
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&report).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn compare_arguments(left_json: &str, right_json: &str) -> PyResult<String> {
    let result = oathkeeper_core::parse(left_json, "json")
        .and_then(|left| {
            oathkeeper_core::parse(right_json, "json")
                .and_then(|right| oathkeeper_core::compare_arguments(&left, &right))
        })
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&result).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn oathkeeper(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(verify, module)?)?;
    module.add_function(wrap_pyfunction!(verify_case, module)?)?;
    module.add_function(wrap_pyfunction!(compile_file, module)?)?;
    module.add_function(wrap_pyfunction!(verify_file, module)?)?;
    module.add_function(wrap_pyfunction!(compare_arguments, module)?)?;
    module.add_function(wrap_pyfunction!(analyze, module)?)?;
    module.add_function(wrap_pyfunction!(analyze_file, module)?)?;
    module.add_function(wrap_pyfunction!(check_context, module)?)?;
    module.add_function(wrap_pyfunction!(assess_backdoor_roth, module)?)?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

#[pyfunction]
fn check_context(bundle_json: &str) -> PyResult<()> {
    oathkeeper_core::parse(bundle_json, "json")
        .and_then(|b| oathkeeper_core::validate_context(&b))
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
#[pyo3(signature = (bundle_json, options_json=None))]
fn analyze(bundle_json: &str, options_json: Option<&str>) -> PyResult<String> {
    let options = match options_json {
        Some(text) => {
            serde_json::from_str(text).map_err(|e| PyValueError::new_err(e.to_string()))?
        }
        None => oathkeeper_core::AnalysisOptions::default(),
    };
    let result = oathkeeper_core::parse(bundle_json, "json")
        .and_then(|b| oathkeeper_core::analyze(&b, &options))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&result).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn analyze_file(path: &str) -> PyResult<String> {
    let result = oathkeeper_core::compile_file(path)
        .and_then(|c| c.bundle())
        .and_then(|b| oathkeeper_core::analyze(&b, &oathkeeper_core::AnalysisOptions::default()))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    serde_json::to_string(&result).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn assess_backdoor_roth(input_json: &str) -> PyResult<String> {
    oathkeeper_core::tax::assess_json(input_json).map_err(|e| PyValueError::new_err(e.to_string()))
}
