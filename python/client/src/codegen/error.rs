// @generated — do not edit by hand.
//! Errors of the Python bindings.

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

/// The client error type.
type ClientError = unitycatalog_client::error::Error;
/// The result of a binding call.
pub type PyUnityCatalogResult<T> = ::core::result::Result<T, PyUnityCatalogError>;
/// An error answer of the server, as the exception classes show it.
pub struct ApiErrorParts {
    /// The error message.
    pub message: String,
    /// The error code of the answer, if any.
    pub code: Option<String>,
    /// The HTTP status of the answer.
    pub status: u16,
    /// The details of the answer.
    pub details: ::std::vec::Vec<::serde_json::Value>,
}

/// Reads the server's error answer out of a client error, for the exception classes.
/// Any other error is raised with its message and causes.
pub trait ApiErrorAnswer: Sized {
    /// The error answer, or the error itself when it is not one.
    fn into_api_error(self) -> ::core::result::Result<ApiErrorParts, Self>;
}

/// The error of a binding call: a client error (raised as a `UnityCatalogError`) or a Python error.
#[derive(Debug)]
pub enum PyUnityCatalogError {
    /// A client error.
    Client(ClientError),
    /// A Python error.
    Python(PyErr),
}

impl ::core::convert::From<ClientError> for PyUnityCatalogError {
    fn from(e: ClientError) -> Self {
        PyUnityCatalogError::Client(e)
    }
}

impl ::core::convert::From<PyErr> for PyUnityCatalogError {
    fn from(e: PyErr) -> Self {
        PyUnityCatalogError::Python(e)
    }
}

impl ::core::convert::From<::url::ParseError> for PyUnityCatalogError {
    fn from(e: ::url::ParseError) -> Self {
        PyUnityCatalogError::Client(ClientError::from(e))
    }
}

impl ::core::convert::From<PyUnityCatalogError> for PyErr {
    fn from(e: PyUnityCatalogError) -> Self {
        match e {
            PyUnityCatalogError::Python(e) => e,
            PyUnityCatalogError::Client(e) => {
                Python::attach(|py| client_exception(py, e).unwrap_or_else(|err| err))
            }
        }
    }
}

/// An error answer of the server, or a request that got no answer.
#[::pyo3::pyclass(
    extends = PyException,
    subclass,
    frozen,
    name = "UnityCatalogError",
    module = "unitycatalog_client._client",
)]
pub struct UnityCatalogError {
    message: String,
    code: Option<String>,
    status: Option<u16>,
    details: Py<PyList>,
}

#[::pyo3::pymethods]
impl UnityCatalogError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> Self {
        Self {
            message,
            code,
            status,
            details: details.map_or_else(|| PyList::empty(py).unbind(), Bound::unbind),
        }
    }

    /// Sets `args` to the message, as `BaseException.__init__` (which takes no keyword
    /// arguments) would.
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    #[allow(unused_variables)]
    fn __init__(
        slf: &Bound<'_, Self>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyResult<()> {
        slf.setattr("args", (message,))
    }

    /// The error message.
    #[getter]
    fn message(&self) -> &str {
        &self.message
    }

    /// The error code of the answer, such as `NOT_FOUND`, if any.
    #[getter]
    fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// The HTTP status of the answer. `None` when the request got no answer.
    #[getter]
    fn status(&self) -> Option<u16> {
        self.status
    }

    /// The details of the answer.
    #[getter]
    fn details<'py>(&self, py: Python<'py>) -> Bound<'py, PyList> {
        self.details.bind(py).clone()
    }

    fn __str__(&self) -> String {
        match (&self.code, self.status) {
            (Some(code), _) => format!("{code}: {}", self.message),
            (None, Some(status)) => format!("HTTP {status}: {}", self.message),
            (None, None) => self.message.clone(),
        }
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let this = slf.get();
        let py = slf.py();
        let mut parts = vec![PyString::new(py, &this.message).repr()?.to_string()];
        if let Some(code) = &this.code {
            parts.push(format!("code={}", PyString::new(py, code).repr()?));
        }
        if let Some(status) = this.status {
            parts.push(format!("status={status}"));
        }
        Ok(format!("{}({})", slf.get_type().name()?, parts.join(", ")))
    }

    /// For `pickle`.
    #[allow(clippy::type_complexity)]
    fn __reduce__<'py>(
        slf: &Bound<'py, Self>,
    ) -> (
        Bound<'py, ::pyo3::types::PyType>,
        (String, Option<String>, Option<u16>, Py<PyList>),
    ) {
        let this = slf.get();
        let args = (
            this.message.clone(),
            this.code.clone(),
            this.status,
            this.details.clone_ref(slf.py()),
        );
        (slf.get_type(), args)
    }
}

/// The server answered `400 Bad Request`: the request is not valid.
#[::pyo3::pyclass(
    extends = UnityCatalogError,
    frozen,
    name = "BadRequestError",
    module = "unitycatalog_client._client",
)]
pub struct BadRequestError;
#[::pyo3::pymethods]
impl BadRequestError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(UnityCatalogError::new(py, message, code, status, details))
            .add_subclass(Self)
    }
}

/// The server answered `401 Unauthorized`: the request has no valid credentials.
#[::pyo3::pyclass(
    extends = UnityCatalogError,
    frozen,
    name = "UnauthenticatedError",
    module = "unitycatalog_client._client",
)]
pub struct UnauthenticatedError;
#[::pyo3::pymethods]
impl UnauthenticatedError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(UnityCatalogError::new(py, message, code, status, details))
            .add_subclass(Self)
    }
}

/// The server answered `403 Forbidden`: the caller may not do this.
#[::pyo3::pyclass(
    extends = UnityCatalogError,
    frozen,
    name = "PermissionDeniedError",
    module = "unitycatalog_client._client",
)]
pub struct PermissionDeniedError;
#[::pyo3::pymethods]
impl PermissionDeniedError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(UnityCatalogError::new(py, message, code, status, details))
            .add_subclass(Self)
    }
}

/// The server answered `404 Not Found`: the resource does not exist.
#[::pyo3::pyclass(
    extends = UnityCatalogError,
    frozen,
    name = "NotFoundError",
    module = "unitycatalog_client._client",
)]
pub struct NotFoundError;
#[::pyo3::pymethods]
impl NotFoundError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(UnityCatalogError::new(py, message, code, status, details))
            .add_subclass(Self)
    }
}

/// The server answered `409 Conflict`: the resource exists or changed.
#[::pyo3::pyclass(
    extends = UnityCatalogError,
    frozen,
    name = "ConflictError",
    module = "unitycatalog_client._client",
)]
pub struct ConflictError;
#[::pyo3::pymethods]
impl ConflictError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(UnityCatalogError::new(py, message, code, status, details))
            .add_subclass(Self)
    }
}

/// The server answered `429 Too Many Requests`.
#[::pyo3::pyclass(
    extends = UnityCatalogError,
    frozen,
    name = "RateLimitError",
    module = "unitycatalog_client._client",
)]
pub struct RateLimitError;
#[::pyo3::pymethods]
impl RateLimitError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(UnityCatalogError::new(py, message, code, status, details))
            .add_subclass(Self)
    }
}

/// The server answered with a `5xx` status: it failed to do the request.
#[::pyo3::pyclass(
    extends = UnityCatalogError,
    frozen,
    name = "ServerError",
    module = "unitycatalog_client._client",
)]
pub struct ServerError;
#[::pyo3::pymethods]
impl ServerError {
    #[new]
    #[pyo3(signature = (message, code = None, status = None, details = None))]
    fn new(
        py: Python<'_>,
        message: String,
        code: Option<String>,
        status: Option<u16>,
        details: Option<Bound<'_, PyList>>,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(UnityCatalogError::new(py, message, code, status, details))
            .add_subclass(Self)
    }
}

/// The exception of a client error: the subclass of its HTTP status.
fn client_exception(py: Python<'_>, e: ClientError) -> PyResult<PyErr> {
    let (message, code, status, details) = match e.into_api_error() {
        Ok(api) => (api.message, api.code, Some(api.status), api.details),
        Err(other) => {
            let mut message = other.to_string();
            let mut cause =
                ::std::error::Error::source(&other).and_then(::std::error::Error::source);
            while let Some(error) = cause {
                message.push_str(": ");
                message.push_str(&error.to_string());
                cause = error.source();
            }
            (message, None, None, ::std::vec::Vec::new())
        }
    };
    let details = details
        .iter()
        .map(|d| json_value(py, d))
        .collect::<PyResult<::std::vec::Vec<_>>>()?;
    let args = (message.clone(),);
    let base = PyClassInitializer::from(UnityCatalogError {
        message,
        code,
        status,
        details: PyList::new(py, details)?.unbind(),
    });
    let exception = match status {
        Some(400u16) => Bound::new(py, base.add_subclass(BadRequestError))?.into_any(),
        Some(401u16) => Bound::new(py, base.add_subclass(UnauthenticatedError))?.into_any(),
        Some(403u16) => Bound::new(py, base.add_subclass(PermissionDeniedError))?.into_any(),
        Some(404u16) => Bound::new(py, base.add_subclass(NotFoundError))?.into_any(),
        Some(409u16) => Bound::new(py, base.add_subclass(ConflictError))?.into_any(),
        Some(429u16) => Bound::new(py, base.add_subclass(RateLimitError))?.into_any(),
        Some(500..=599) => Bound::new(py, base.add_subclass(ServerError))?.into_any(),
        _ => Bound::new(py, base)?.into_any(),
    };
    exception.setattr("args", args)?;
    Ok(PyErr::from_value(exception))
}

/// A JSON value as a Python object.
fn json_value<'py>(py: Python<'py>, value: &::serde_json::Value) -> PyResult<Bound<'py, PyAny>> {
    use ::serde_json::Value;
    Ok(match value {
        Value::Null => py.None().into_bound(py),
        Value::Bool(b) => b.into_pyobject(py)?.to_owned().into_any(),
        Value::Number(n) => match (n.as_i64(), n.as_u64()) {
            (Some(i), _) => i.into_pyobject(py)?.into_any(),
            (None, Some(u)) => u.into_pyobject(py)?.into_any(),
            (None, None) => n.as_f64().unwrap_or(f64::NAN).into_pyobject(py)?.into_any(),
        },
        Value::String(s) => PyString::new(py, s).into_any(),
        Value::Array(items) => PyList::new(
            py,
            items
                .iter()
                .map(|item| json_value(py, item))
                .collect::<PyResult<::std::vec::Vec<_>>>()?,
        )?
        .into_any(),
        Value::Object(map) => {
            let dict = PyDict::new(py);
            for (key, item) in map {
                dict.set_item(key, json_value(py, item)?)?;
            }
            dict.into_any()
        }
    })
}
