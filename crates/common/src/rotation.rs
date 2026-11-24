//! KEK rotation across every store that seals values: counts and sweeps.
//!
//! The server seals values in more than one place (the object graph's sensitive
//! fields, storage credentials, recipient tokens). Each envelope records the id
//! of the KEK that wrapped its data key, so rotating a KEK means:
//!
//! 1. make the new KEK active (new writes are sealed under it);
//! 2. [`sweep`] every [`SealedStore`]: re-wrap what is still under an older KEK;
//! 3. check each store's [`KekUsage`] until the old KEK counts zero;
//! 4. retire the old KEK.
//!
//! Reads never re-wrap, so step 2 is the only thing that moves values. A sweep
//! is idempotent: running it again resumes an interrupted one, and values
//! already moved count as [`already_active`](RewrapReport::already_active).
//!
//! These types carry no key material and serialize for an admin API.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// How many sealed values each KEK wraps, counted from envelope headers
/// without decrypting anything.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KekUsage {
    /// Values per KEK id. KEKs that wrap nothing are absent.
    pub by_kek: BTreeMap<String, u64>,
    /// Values whose header could not be read (corrupt, or an unknown format).
    pub unreadable: u64,
}

impl KekUsage {
    /// Values sealed under `kek_id`.
    pub fn count(&self, kek_id: &str) -> u64 {
        self.by_kek.get(kek_id).copied().unwrap_or(0)
    }

    /// All values counted, unreadable ones included.
    pub fn total(&self) -> u64 {
        self.by_kek.values().sum::<u64>() + self.unreadable
    }

    /// Count one value: `Some(kek_id)` from its header, `None` if unreadable.
    pub fn record(&mut self, kek_id: Option<String>) {
        match kek_id {
            Some(id) => *self.by_kek.entry(id).or_default() += 1,
            None => self.unreadable += 1,
        }
    }

    /// Values sealed under any KEK other than `active`, unreadable ones included:
    /// what a sweep still has to move before an old KEK can be retired.
    pub fn not_under(&self, active: &str) -> u64 {
        self.total() - self.count(active)
    }
}

#[cfg(feature = "sealing")]
impl From<olai_store::KekUsage> for KekUsage {
    fn from(usage: olai_store::KekUsage) -> Self {
        Self {
            by_kek: usage.by_kek,
            unreadable: usage.unreadable,
        }
    }
}

/// Why a value could not be re-wrapped. It stays under its old KEK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewrapFailureKind {
    /// The envelope header could not be read.
    Malformed,
    /// The key provider refused to unwrap or wrap (e.g. the KEK is retired).
    KeyProvider,
    /// The store failed to write the re-wrapped value.
    Store,
}

/// A value the sweep could not move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewrapFailure {
    /// The row or object id.
    pub id: String,
    /// The KEK the value is sealed under, if its header was readable.
    pub kek_id: Option<String>,
    pub kind: RewrapFailureKind,
    /// The error, for the operator. Never holds key material.
    pub message: String,
}

/// The outcome of re-wrapping one store.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewrapReport {
    /// Values moved to the active KEK.
    pub rewrapped: u64,
    /// Values already under the active KEK.
    pub already_active: u64,
    /// Values replaced by a concurrent writer between the sweep's read and its
    /// write. The writer's value is kept; it is sealed under the active KEK.
    pub superseded: u64,
    /// Values left under their old KEK. The sweep went on past them.
    pub failed: Vec<RewrapFailure>,
    /// The store's KEK usage after the sweep.
    pub kek_usage: KekUsage,
}

#[cfg(feature = "sealing")]
impl<L: fmt::Display> From<olai_store::RewrapReport<L>> for RewrapReport {
    fn from(report: olai_store::RewrapReport<L>) -> Self {
        Self {
            rewrapped: report.rewrapped,
            already_active: report.already_active,
            superseded: report.superseded,
            failed: report
                .failed
                .into_iter()
                .map(|f| {
                    let (kind, message) = match f.kind {
                        olai_store::RewrapFailureKind::Malformed => (
                            RewrapFailureKind::Malformed,
                            "malformed envelope".to_string(),
                        ),
                        olai_store::RewrapFailureKind::KeyProvider => (
                            RewrapFailureKind::KeyProvider,
                            "the key provider refused the KEK".to_string(),
                        ),
                        olai_store::RewrapFailureKind::Store(kind) => {
                            (RewrapFailureKind::Store, format!("store error: {kind}"))
                        }
                    };
                    RewrapFailure {
                        id: format!("{}/{}", f.label, f.id),
                        kek_id: f.kek_id,
                        kind,
                        message,
                    }
                })
                .collect(),
            kek_usage: report.kek_usage.into(),
        }
    }
}

/// An error that stopped a whole store's count or sweep (e.g. a listing failed).
pub type SealedStoreError = Box<dyn std::error::Error + Send + Sync>;

/// A store that seals values under a KEK and can count and re-wrap them.
#[async_trait::async_trait]
pub trait SealedStore: Send + Sync {
    /// A stable name for reports, e.g. `"storage_credentials"`.
    fn name(&self) -> &str;

    /// How many values each KEK wraps. Reads headers only; decrypts nothing.
    async fn kek_usage(&self) -> Result<KekUsage, SealedStoreError>;

    /// Re-wrap every value under the active KEK. A value that fails is reported
    /// and skipped; only an error that stops the whole store returns `Err`.
    async fn rewrap(&self) -> Result<RewrapReport, SealedStoreError>;
}

/// One store's part of a [`sweep`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreSweep {
    /// The store's [`name`](SealedStore::name).
    pub store: String,
    /// KEK usage before the sweep, if it could be counted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<KekUsage>,
    /// The sweep's outcome; its `kek_usage` is the usage after.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<RewrapReport>,
    /// What stopped this store, if anything did. The other stores still ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Count, then re-wrap, each store in turn.
///
/// A store that fails as a whole gets an [`error`](StoreSweep::error) and the
/// sweep moves on to the next one.
pub async fn sweep(stores: &[&dyn SealedStore]) -> Vec<StoreSweep> {
    let mut out = Vec::with_capacity(stores.len());
    for store in stores {
        let mut entry = StoreSweep {
            store: store.name().to_string(),
            before: None,
            report: None,
            error: None,
        };
        match store.kek_usage().await {
            Ok(usage) => entry.before = Some(usage),
            Err(e) => entry.error = Some(e.to_string()),
        }
        match store.rewrap().await {
            Ok(report) => entry.report = Some(report),
            Err(e) => entry.error = Some(e.to_string()),
        }
        out.push(entry);
    }
    out
}

/// A paged view of a table of sealed values, for [`rows::kek_usage`] and
/// [`rows::rewrap`].
#[cfg(feature = "sealing")]
pub mod rows {
    use olai_store::encryption::{EnvelopeEncryptor, envelope_kek_id};

    use super::{KekUsage, RewrapFailure, RewrapFailureKind, RewrapReport};

    /// Rows read per page.
    pub const PAGE_SIZE: usize = 256;

    /// A table of sealed values, keyed by a string id.
    #[async_trait::async_trait]
    pub trait SealedRows: Send + Sync {
        type Error: std::fmt::Display + Send;

        /// Up to `limit` `(id, sealed)` rows with ids after `after`, in id order.
        async fn page(
            &self,
            after: Option<&str>,
            limit: usize,
        ) -> Result<Vec<(String, Vec<u8>)>, Self::Error>;

        /// Replace row `id`'s value with `fresh` if it still is `current`.
        /// Returns whether it was replaced.
        async fn replace(
            &self,
            id: &str,
            current: &[u8],
            fresh: &[u8],
        ) -> Result<bool, Self::Error>;
    }

    /// Count the rows per KEK from their envelope headers, page by page.
    pub async fn kek_usage<R: SealedRows + ?Sized>(rows: &R) -> Result<KekUsage, R::Error> {
        let mut usage = KekUsage::default();
        let mut after: Option<String> = None;
        loop {
            let page = rows.page(after.as_deref(), PAGE_SIZE).await?;
            let done = page.len() < PAGE_SIZE;
            for (_, sealed) in &page {
                usage.record(envelope_kek_id(sealed).ok());
            }
            match page.into_iter().last() {
                Some((id, _)) if !done => after = Some(id),
                _ => return Ok(usage),
            }
        }
    }

    /// Re-wrap every row under the active KEK, page by page, with a
    /// compare-and-swap write so a concurrent update is never overwritten.
    ///
    /// A row that fails is recorded in [`RewrapReport::failed`] and skipped.
    /// Only a failed page read stops the sweep.
    pub async fn rewrap<R: SealedRows + ?Sized>(
        rows: &R,
        encryptor: &EnvelopeEncryptor,
    ) -> Result<RewrapReport, R::Error> {
        let mut report = RewrapReport::default();
        let mut after: Option<String> = None;
        loop {
            let page = rows.page(after.as_deref(), PAGE_SIZE).await?;
            let done = page.len() < PAGE_SIZE;
            for (id, sealed) in &page {
                let kek_id = envelope_kek_id(sealed).ok();
                let fresh = match encryptor.rewrap(sealed).await {
                    Ok(None) => {
                        report.already_active += 1;
                        continue;
                    }
                    Ok(Some(fresh)) => fresh,
                    Err(e) => {
                        let kind = if kek_id.is_none() {
                            RewrapFailureKind::Malformed
                        } else {
                            RewrapFailureKind::KeyProvider
                        };
                        report.failed.push(RewrapFailure {
                            id: id.clone(),
                            kek_id,
                            kind,
                            message: e.to_string(),
                        });
                        continue;
                    }
                };
                match rows.replace(id, sealed, &fresh).await {
                    Ok(true) => report.rewrapped += 1,
                    Ok(false) => report.superseded += 1,
                    Err(e) => report.failed.push(RewrapFailure {
                        id: id.clone(),
                        kek_id,
                        kind: RewrapFailureKind::Store,
                        message: e.to_string(),
                    }),
                }
            }
            match page.into_iter().last() {
                Some((id, _)) if !done => after = Some(id),
                _ => break,
            }
        }
        report.kek_usage = kek_usage(rows).await?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_counts() {
        let mut usage = KekUsage::default();
        usage.record(Some("k@v1".into()));
        usage.record(Some("k@v1".into()));
        usage.record(Some("k@v2".into()));
        usage.record(None);
        assert_eq!(usage.count("k@v1"), 2);
        assert_eq!(usage.count("k@v3"), 0);
        assert_eq!(usage.total(), 4);
        assert_eq!(usage.not_under("k@v2"), 3);
    }

    struct Failing;

    #[async_trait::async_trait]
    impl SealedStore for Failing {
        fn name(&self) -> &str {
            "failing"
        }
        async fn kek_usage(&self) -> Result<KekUsage, SealedStoreError> {
            Err("listing failed".into())
        }
        async fn rewrap(&self) -> Result<RewrapReport, SealedStoreError> {
            Err("listing failed".into())
        }
    }

    struct Empty;

    #[async_trait::async_trait]
    impl SealedStore for Empty {
        fn name(&self) -> &str {
            "empty"
        }
        async fn kek_usage(&self) -> Result<KekUsage, SealedStoreError> {
            Ok(KekUsage::default())
        }
        async fn rewrap(&self) -> Result<RewrapReport, SealedStoreError> {
            Ok(RewrapReport::default())
        }
    }

    #[tokio::test]
    async fn a_failing_store_does_not_stop_the_sweep() {
        let report = sweep(&[&Failing, &Empty]).await;
        assert_eq!(report.len(), 2);
        assert_eq!(report[0].error.as_deref(), Some("listing failed"));
        assert!(report[0].report.is_none());
        assert_eq!(report[1].store, "empty");
        assert!(report[1].error.is_none());
        assert_eq!(report[1].report, Some(RewrapReport::default()));
    }
}
