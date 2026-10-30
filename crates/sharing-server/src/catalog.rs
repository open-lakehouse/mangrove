//! [`StoreCatalog`]: the recipient-facing [`ShareCatalog`] over the share store
//! and the source catalogs.
//!
//! A share's [`ShareObject`]s name assets in a source catalog. They are exposed
//! to recipients under their `shared_as` names (default: the source name minus
//! its catalog), and resolved through the [`SourceRegistry`] at request time — the
//! source catalog stays the source of truth for locations and formats:
//!
//! | object kind      | `name`            | exposed as                 |
//! |-------------------|-------------------|----------------------------|
//! | `TABLE`           | `cat.schema.tbl`  | `schema.tbl` (or `shared_as`) |
//! | `SCHEMA`          | `cat.schema`      | every table of the schema, under `schema` (or `shared_as`) |
//! | `VOLUME`          | `cat.schema.vol`  | `schema.vol` (or `string_shared_as`) |
//! | `AGENT_SKILL`     | `cat.schema.sk`   | `schema.sk` (or `string_shared_as`), resolved as a volume |
//!
//! Like the Databricks API, non-table objects take their alias from
//! `string_shared_as`; `shared_as` is still honoured for them as a fallback.
//!
//! A recipient sees only the shares granted to it; any other share is reported
//! as not found.

use std::sync::Arc;

use async_trait::async_trait;
use open_sharing_api::auth::Recipient;
use open_sharing_api::catalog::{
    AssetKind, AssetRef, Page, PageRequest, ResolvedLocation, ShareCatalog, ShareInfo, SharedAsset,
    SharedObject, SharingAction,
};
use open_sharing_api::service::paginate;

use crate::source::{SourceRegistry, split_name};
use crate::store::{ShareObject, ShareObjectKind, ShareRecord, ShareStore};
use crate::{Error, Result};

/// A shared object, with the source-catalog name it resolves through.
#[derive(Debug, Clone)]
struct Exposed {
    object: SharedObject,
    source_name: String,
    share_object: ShareObject,
}

/// The store-backed share catalog. See the module docs.
pub struct StoreCatalog {
    store: Arc<dyn ShareStore>,
    sources: SourceRegistry,
}

impl StoreCatalog {
    pub fn new(store: Arc<dyn ShareStore>, sources: SourceRegistry) -> Self {
        Self { store, sources }
    }

    /// The share's objects as exposed to recipients, in share order.
    async fn expose(&self, share: &ShareRecord) -> Result<Vec<Exposed>> {
        let mut exposed = Vec::new();
        for share_object in &share.objects {
            let kind = match share_object.kind {
                ShareObjectKind::Table => AssetKind::Table,
                ShareObjectKind::Volume => AssetKind::Volume,
                ShareObjectKind::AgentSkill => AssetKind::AgentSkill,
                ShareObjectKind::Schema => {
                    let [catalog, schema] = split_name::<2>(&share_object.name)?;
                    let shared_schema = share_object.shared_as.as_deref().unwrap_or(schema);
                    let source = self.sources.for_name(&share_object.name)?;
                    for table in source.source.list_tables(catalog, schema).await? {
                        exposed.push(Exposed {
                            object: SharedObject {
                                schema: shared_schema.to_string(),
                                name: table.name.clone(),
                                kind: AssetKind::Table,
                                id: table.id.clone(),
                            },
                            source_name: table.full_name,
                            share_object: share_object.clone(),
                        });
                    }
                    continue;
                }
            };
            let (schema, name) = shared_as(share_object)?;
            exposed.push(Exposed {
                object: SharedObject {
                    schema,
                    name,
                    kind,
                    id: None,
                },
                source_name: share_object.name.clone(),
                share_object: share_object.clone(),
            });
        }
        Ok(exposed)
    }
}

/// The `(schema, name)` a table / volume / skill object is exposed under.
pub fn shared_as(object: &ShareObject) -> Result<(String, String)> {
    let alias = match object.kind {
        ShareObjectKind::Volume | ShareObjectKind::AgentSkill => object
            .extra
            .get("string_shared_as")
            .and_then(|v| v.as_str())
            .or(object.shared_as.as_deref()),
        ShareObjectKind::Table | ShareObjectKind::Schema => object.shared_as.as_deref(),
    };
    let shared = match alias {
        Some(shared) => shared.to_string(),
        None => {
            let [_, schema, name] = split_name::<3>(&object.name)?;
            format!("{schema}.{name}")
        }
    };
    let [schema, name] = split_name::<2>(&shared)?;
    Ok((schema.to_string(), name.to_string()))
}

fn info(share: &ShareRecord) -> ShareInfo {
    ShareInfo {
        name: share.name.clone(),
        id: Some(share.id.clone()),
        comment: share.comment.clone(),
    }
}

#[async_trait]
impl ShareCatalog<Recipient> for StoreCatalog {
    async fn authorize(&self, action: SharingAction<'_>, cx: &Recipient) -> Result<()> {
        match action {
            SharingAction::ListShares => Ok(()),
            SharingAction::ReadShare { share } => {
                if self.store.is_granted(share, &cx.id).await? {
                    Ok(())
                } else {
                    // Do not reveal whether an ungranted share exists.
                    Err(Error::NotFound)
                }
            }
        }
    }

    async fn list_shares(&self, page: PageRequest, cx: &Recipient) -> Result<Page<ShareInfo>> {
        let shares = self.store.shares_of(&cx.id).await?;
        paginate(
            shares.iter().map(info).collect(),
            page.max_results.map(|n| n.min(i32::MAX as usize) as i32),
            page.page_token.as_deref(),
        )
    }

    async fn get_share(&self, share: &str, _cx: &Recipient) -> Result<ShareInfo> {
        Ok(info(&self.store.get_share(share).await?))
    }

    async fn list_objects(&self, share: &str, _cx: &Recipient) -> Result<Vec<SharedObject>> {
        let share = self.store.get_share(share).await?;
        Ok(self
            .expose(&share)
            .await?
            .into_iter()
            .map(|e| e.object)
            .collect())
    }

    async fn resolve(&self, asset: &AssetRef, _cx: &Recipient) -> Result<SharedAsset> {
        let share = self.store.get_share(&asset.share).await?;
        let exposed = self
            .expose(&share)
            .await?
            .into_iter()
            .find(|e| {
                e.object.kind == asset.kind
                    && e.object.schema == asset.schema
                    && e.object.name == asset.name
            })
            .ok_or(Error::NotFound)?;
        let source = self.sources.for_name(&exposed.source_name)?;
        let history_shared = exposed.share_object.history_shared;
        let start_version = exposed
            .share_object
            .start_version
            .map(|v| u64::try_from(v).unwrap_or_default());
        Ok(match asset.kind {
            AssetKind::Table => {
                let table = source.source.get_table(&exposed.source_name).await?;
                SharedAsset {
                    reference: asset.clone(),
                    id: table.id,
                    location: ResolvedLocation::parse(&table.location)?,
                    format: Some(table.format),
                    history_shared,
                    start_version,
                    source_name: Some(table.full_name),
                }
            }
            AssetKind::Volume | AssetKind::AgentSkill => {
                let volume = source.source.get_volume(&exposed.source_name).await?;
                SharedAsset {
                    reference: asset.clone(),
                    id: volume.id,
                    location: ResolvedLocation::parse(&volume.location)?,
                    format: None,
                    history_shared: false,
                    start_version: None,
                    source_name: Some(volume.full_name),
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(kind: ShareObjectKind, extra: serde_json::Value) -> ShareObject {
        let mut object = ShareObject::new(kind, "cat.sales.orders");
        if let serde_json::Value::Object(extra) = extra {
            object.extra = extra;
        }
        object
    }

    #[test]
    fn test_shared_as_defaults_to_source_name_without_catalog() {
        let table = object(ShareObjectKind::Table, serde_json::json!({}));
        assert_eq!(
            shared_as(&table).unwrap(),
            ("sales".into(), "orders".into())
        );
    }

    #[test]
    fn test_shared_as_reads_string_shared_as_for_volumes_only() {
        let alias = serde_json::json!({ "string_shared_as": "shared.files" });
        let volume = object(ShareObjectKind::Volume, alias.clone());
        assert_eq!(
            shared_as(&volume).unwrap(),
            ("shared".into(), "files".into())
        );

        // Table-like objects keep using `shared_as`.
        let table = object(ShareObjectKind::Table, alias);
        assert_eq!(
            shared_as(&table).unwrap(),
            ("sales".into(), "orders".into())
        );

        // `shared_as` remains a fallback for non-table objects.
        let mut legacy = object(ShareObjectKind::Volume, serde_json::json!({}));
        legacy.shared_as = Some("legacy.files".into());
        assert_eq!(
            shared_as(&legacy).unwrap(),
            ("legacy".into(), "files".into())
        );
    }
}
