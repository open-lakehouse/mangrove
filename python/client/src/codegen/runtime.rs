// @generated — do not edit by hand.
//! Tokio runtime and pagination plumbing shared by the generated Python clients.
use futures::stream::{BoxStream, Stream, StreamExt};
use pyo3::prelude::*;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
/// How often a blocking call wakes up to let Python handle pending signals (Ctrl-C).
const SIGNAL_CHECK_INTERVAL: Duration = Duration::from_millis(100);
/// The process-wide Tokio runtime shared by every generated client.
///
/// Built lazily on first use, so importing the extension (or forking before the first
/// call) never starts runtime threads. A forked child must not reuse a runtime its
/// parent already started.
pub fn runtime() -> &'static ::tokio::runtime::Runtime {
    static RUNTIME: ::std::sync::OnceLock<::tokio::runtime::Runtime> = ::std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        ::tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name(concat!(env!("CARGO_PKG_NAME"), "-tokio"))
            .build()
            .expect("failed to build the Tokio runtime")
    })
}
/// Drive `fut` to completion on [`runtime`] with the GIL released.
///
/// Every [`SIGNAL_CHECK_INTERVAL`] the GIL is briefly re-acquired to run Python signal
/// handlers; if one raises (e.g. `KeyboardInterrupt`), `fut` is dropped, cancelling the
/// in-flight request, and the exception is returned.
pub fn block_on<F>(py: Python<'_>, fut: F) -> PyResult<F::Output>
where
    F: Future + Send,
    F::Output: Send,
{
    let runtime = runtime();
    let mut fut = ::std::pin::pin!(fut);
    loop {
        let polled = py.detach(|| {
            runtime.block_on(async {
                ::tokio::time::timeout(SIGNAL_CHECK_INTERVAL, &mut fut)
                    .await
                    .ok()
            })
        });
        if let Some(output) = polled {
            return Ok(output);
        }
        py.check_signals()?;
    }
}
/// The type-erased item stream behind the list pagers: each item is already converted to
/// its Python wrapper object.
type ItemStream = BoxStream<'static, PyResult<Py<PyAny>>>;
/// A lazy, blocking iterator over a paginated list endpoint.
///
/// Pages are fetched on demand as the iterator advances; each fetch releases the GIL.
#[pyclass(name = "ListIterator")]
pub struct ListIterator {
    stream: Arc<::tokio::sync::Mutex<ItemStream>>,
}
impl ListIterator {
    pub fn new(stream: impl Stream<Item = PyResult<Py<PyAny>>> + Send + 'static) -> Self {
        Self {
            stream: Arc::new(::tokio::sync::Mutex::new(stream.fuse().boxed())),
        }
    }
}
#[pymethods]
impl ListIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let stream = Arc::clone(&self.stream);
        block_on(py, async move { stream.lock().await.next().await })?.transpose()
    }
}
