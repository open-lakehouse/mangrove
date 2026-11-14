//! Captures the server's audit events (target `unitycatalog::audit`) for tests.
//! Shared by integration tests (`mod support`) and unit tests (`#[path]`).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::{Layer, Registry};

/// One audit event's fields, all formatted as strings.
pub type AuditEvent = BTreeMap<String, String>;

/// The audit events emitted while a [`capture`] guard is alive on this thread.
#[derive(Clone, Default)]
pub struct Captured(Arc<Mutex<Vec<AuditEvent>>>);

impl Captured {
    /// The events so far.
    pub fn events(&self) -> Vec<AuditEvent> {
        self.0.lock().unwrap().clone()
    }
}

struct AuditLayer(Captured);

struct Fields<'a>(&'a mut AuditEvent);

impl Visit for Fields<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().into(), format!("{value:?}"));
    }
}

impl<S: tracing::Subscriber> Layer<S> for AuditLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _: Context<'_, S>) {
        if event.metadata().target() != "unitycatalog::audit" {
            return;
        }
        let mut fields = AuditEvent::new();
        event.record(&mut Fields(&mut fields));
        (self.0).0.lock().unwrap().push(fields);
    }
}

/// Capture audit events on this thread (a current-thread `#[tokio::test]`)
/// until the guard drops.
pub fn capture() -> (Captured, tracing::subscriber::DefaultGuard) {
    let captured = Captured::default();
    let subscriber = Registry::default().with(AuditLayer(captured.clone()));
    (captured, tracing::subscriber::set_default(subscriber))
}
