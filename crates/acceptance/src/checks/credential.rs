//! Storage credential and external-location coverage (extended; needs cloud identity).

use futures::StreamExt;
use unitycatalog_common::credentials::v1::{AwsIamRoleConfig, AzureManagedIdentity, Purpose};

use super::{unique, with_cleanup};
use crate::conformance::skip;
use crate::{AcceptanceResult, JourneyContext};

/// Read a configured cloud identity from the environment, or `None`.
fn cloud_identity() -> Option<CloudIdentity> {
    if let Ok(role_arn) = std::env::var("UC_TEST_AWS_ROLE_ARN") {
        Some(CloudIdentity::Aws(role_arn))
    } else if let Ok(connector) = std::env::var("UC_TEST_AZURE_ACCESS_CONNECTOR_ID") {
        Some(CloudIdentity::Azure(connector))
    } else {
        None
    }
}

enum CloudIdentity {
    Aws(String),
    Azure(String),
}

/// Storage credential: create → get → list → update comment → delete.
/// Self-skips unless `UC_TEST_AWS_ROLE_ARN` or `UC_TEST_AZURE_ACCESS_CONNECTOR_ID` is set.
pub async fn credential_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let Some(identity) = cloud_identity() else {
        return Err(skip(
            "set UC_TEST_AWS_ROLE_ARN or UC_TEST_AZURE_ACCESS_CONNECTOR_ID to exercise credentials",
        ));
    };

    let credential = unique("conf_cred");
    with_cleanup(
        || async {
            let mut builder = ctx
                .client()
                .create_credential(&credential, Purpose::Storage)
                .with_comment("conformance credential".to_string());
            builder = match identity {
                CloudIdentity::Aws(role_arn) => builder.with_aws_iam_role(AwsIamRoleConfig {
                    role_arn,
                    ..Default::default()
                }),
                CloudIdentity::Azure(access_connector_id) => {
                    builder.with_azure_managed_identity(AzureManagedIdentity {
                        access_connector_id,
                        ..Default::default()
                    })
                }
            };
            let created = builder.await?;
            assert_eq!(created.name, credential);

            let fetched = ctx.client().credential(&credential).get().await?;
            assert_eq!(fetched.name, credential);

            let listed: Vec<_> = ctx
                .client()
                .list_credentials()
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                listed.iter().any(|c| c.name == credential),
                "credential missing from listing"
            );

            ctx.client()
                .credential(&credential)
                .update()
                .with_comment("updated comment".to_string())
                .await?;
            Ok(())
        },
        || async {
            let _ = ctx.client().credential(&credential).delete().await;
        },
    )
    .await
}

/// External location backed by a storage credential: create → list → delete.
/// Self-skips without a cloud storage root.
pub async fn external_location_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    if !ctx.storage_root.starts_with("s3://")
        && !ctx.storage_root.starts_with("abfss://")
        && !ctx.storage_root.starts_with("gs://")
    {
        return Err(skip(
            "external location needs a cloud storage root (UC_INTEGRATION_STORAGE_ROOT)",
        ));
    }

    let credential = unique("conf_el_cred");
    let ext_loc = unique("conf_el");
    let url = format!(
        "{}/ext-loc/{ext_loc}/",
        ctx.storage_root.trim_end_matches('/')
    );
    with_cleanup(
        || async {
            ctx.client()
                .create_credential(&credential, Purpose::Storage)
                .await?;
            let created = ctx
                .client()
                .create_external_location(&ext_loc, &url, &credential)
                .with_comment("conformance external location".to_string())
                .await?;
            assert_eq!(created.name, ext_loc);

            let listed: Vec<_> = ctx
                .client()
                .list_external_locations()
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                listed.iter().any(|l| l.name == ext_loc),
                "external location missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().external_location(&ext_loc).delete().await;
            let _ = ctx.client().credential(&credential).delete().await;
        },
    )
    .await
}
