// @generated — do not edit by hand.
//! The Tokio runtime of the bindings, made on first use.

use std::sync::OnceLock;
use tokio::runtime::{Builder, Runtime};

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
