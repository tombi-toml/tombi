mod error;

use error::to_py_err;
use pyo3::{exceptions::PyValueError, prelude::*};

fn deserialize_options(options: Option<&Bound<'_, PyAny>>) -> PyResult<tombi_lib::Options> {
    match options {
        Some(options) => pythonize::depythonize(options)
            .map_err(|error| PyValueError::new_err(error.to_string())),
        None => Ok(tombi_lib::Options::default()),
    }
}

/// Format a TOML document.
#[pyfunction]
#[pyo3(signature = (source, source_path, options=None))]
fn format(
    source: String,
    source_path: String,
    options: Option<&Bound<'_, PyAny>>,
) -> PyResult<tombi_lib::FormatResult> {
    let options = deserialize_options(options)?;
    tombi_lib::format_sync(source, source_path, options).map_err(to_py_err)
}

/// Lint a TOML document.
#[pyfunction]
#[pyo3(signature = (source, source_path, options=None))]
fn lint(
    source: String,
    source_path: String,
    options: Option<&Bound<'_, PyAny>>,
) -> PyResult<tombi_lib::LintResult> {
    let options = deserialize_options(options)?;
    tombi_lib::lint_sync(source, source_path, options).map_err(to_py_err)
}

/// Format a TOML document, returning an awaitable.
#[cfg(feature = "async")]
#[pyfunction]
#[pyo3(signature = (source, source_path, options=None))]
fn format_async<'py>(
    py: Python<'py>,
    source: String,
    source_path: String,
    options: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyAny>> {
    let options = deserialize_options(options)?;
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        tombi_lib::format_async(source, source_path, options)
            .await
            .map_err(to_py_err)
    })
}

/// Lint a TOML document, returning an awaitable.
#[cfg(feature = "async")]
#[pyfunction]
#[pyo3(signature = (source, source_path, options=None))]
fn lint_async<'py>(
    py: Python<'py>,
    source: String,
    source_path: String,
    options: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyAny>> {
    let options = deserialize_options(options)?;
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        tombi_lib::lint_async(source, source_path, options)
            .await
            .map_err(to_py_err)
    })
}

#[pymodule]
fn _tombi_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format, m)?)?;
    m.add_function(wrap_pyfunction!(lint, m)?)?;
    #[cfg(feature = "async")]
    m.add_function(wrap_pyfunction!(format_async, m)?)?;
    #[cfg(feature = "async")]
    m.add_function(wrap_pyfunction!(lint_async, m)?)?;
    m.add_class::<tombi_lib::FormatResult>()?;
    m.add_class::<tombi_lib::LintResult>()?;
    m.add_class::<tombi_lib::Diagnostic>()?;
    m.add("TombiError", m.py().get_type::<error::TombiError>())?;
    m.add(
        "TombiConfigError",
        m.py().get_type::<error::TombiConfigError>(),
    )?;
    m.add(
        "TombiSchemaError",
        m.py().get_type::<error::TombiSchemaError>(),
    )?;
    Ok(())
}
