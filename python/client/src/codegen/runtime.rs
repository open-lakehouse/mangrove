// @generated — do not edit by hand.
//! The Tokio runtime of the bindings, made on first use.

use pyo3::prelude::*;
use std::future::Future;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

/// How often a blocking call wakes up to let Python handle pending signals (Ctrl-C).
const SIGNAL_CHECK_INTERVAL: Duration = Duration::from_millis(100);
/// The runtime. It panics in a forked child, where Tokio's thread pool does not work.
pub fn rt() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    static PID: OnceLock<u32> = OnceLock::new();
    let pid = std::process::id();
    let runtime_pid = *PID.get_or_init(|| pid);
    if pid != runtime_pid {
        panic!(
            "Forked process detected: current PID is {pid} but the tokio runtime was \
             created by {runtime_pid}. The tokio runtime does not support forked processes \
             (https://github.com/tokio-rs/tokio/issues/4301). When using Python \
             multiprocessing, set the start method to 'spawn' or 'forkserver'."
        );
    }
    RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build the tokio runtime")
    })
}

/// Drive `fut` to completion on [`rt`] with the GIL released.
///
/// Every [`SIGNAL_CHECK_INTERVAL`] the GIL is re-acquired to run the Python signal
/// handlers; if one raises (such as `KeyboardInterrupt`), `fut` is dropped, which cancels
/// the request, and the exception is returned.
pub fn block_on<F>(py: Python<'_>, fut: F) -> PyResult<F::Output>
where
    F: Future + Send,
    F::Output: Send,
{
    let runtime = rt();
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
