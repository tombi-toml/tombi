use pyo3::{PyErr, create_exception, exceptions::PyException};

// Named `tombi_lib::Error::NAME`, like the Node.js/wasm bindings' JS errors.
create_exception!(_tombi_lib, TombiError, PyException);

pub(crate) fn to_py_err(error: tombi_lib::Error) -> PyErr {
    TombiError::new_err(error.to_string())
}
