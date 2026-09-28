use pyo3::{PyErr, create_exception, exceptions::PyException};

// `TombiError` is only a base class for `except TombiError:`; every raised
// error is one of its subclasses, named after `tombi_lib::Error::name`.
create_exception!(_tombi_lib, TombiError, PyException);
create_exception!(_tombi_lib, TombiConfigError, TombiError);
create_exception!(_tombi_lib, TombiSchemaError, TombiError);
create_exception!(_tombi_lib, TombiIOError, TombiError);

pub(crate) fn to_py_err(error: tombi_lib::Error) -> PyErr {
    match error {
        tombi_lib::Error::Io(_) => TombiIOError::new_err(error.to_string()),
        tombi_lib::Error::Config(_) => TombiConfigError::new_err(error.to_string()),
        tombi_lib::Error::Schema(_) => TombiSchemaError::new_err(error.to_string()),
    }
}
