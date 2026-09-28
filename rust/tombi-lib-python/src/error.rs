use pyo3::{PyErr, create_exception, exceptions::PyException};

create_exception!(_tombi_lib, TombiError, PyException);
create_exception!(_tombi_lib, TombiConfigError, TombiError);
create_exception!(_tombi_lib, TombiSchemaError, TombiError);

pub(crate) fn to_py_err(error: tombi_lib::Error) -> PyErr {
    match error {
        tombi_lib::Error::Io(_) => TombiError::new_err(error.to_string()),
        tombi_lib::Error::Config(_) => TombiConfigError::new_err(error.to_string()),
        tombi_lib::Error::Schema(_) => TombiSchemaError::new_err(error.to_string()),
    }
}
